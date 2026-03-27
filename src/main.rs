mod checkout;
mod i18n;
mod message;
mod product;
mod views;

use crate::checkout::{CartItem, CheckoutSession, ConnectPayload, SyncCartPayload};
use crate::i18n::I18n;
use crate::message::Message;
use crate::product::{Product, ProductImage, ProductsResponse};

use iced::widget::{button, column, container, image, text};
use iced::{Element, Length, Task, Theme, application};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use views::intro::welcome_view;
use views::session::session_view;

fn main() -> iced::Result {
    application(SelfCheckout::new, update, view)
        .theme(app_theme)
        .run()
}

#[derive(Debug, Clone, Default)]
enum Screen {
    #[default]
    Connecting,
    Welcome,
    Session,
}

struct SelfCheckout {
    screen: Screen,
    i18n: I18n,
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
    checkout_session: Option<CheckoutSession>,
    products: Vec<Product>,
    product_images: HashMap<String, ProductImage>,
    search: String,
    status: String,
    loading_products: bool,
    cart: Vec<CartItem>,
    selected_product: Option<Product>,
    quantity_input: String,
    quantity_error: String,
    measuring_weight: bool,
    connection_failed: bool,
    recovering_connection: bool,
    manual_reconnect_available: bool,
}

impl SelfCheckout {
    fn new() -> (Self, Task<Message>) {
        let _ = dotenvy::dotenv();

        let language = env::var("DEFAULT_LANG").unwrap_or_else(|_| "en".to_string());
        let api_base_url =
            env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let counter_id = env::var("CHECKOUT_COUNTER_ID").unwrap_or_default();
        let counter_password = env::var("CHECKOUT_COUNTER_PASSWORD").unwrap_or_default();
        let client_id = load_or_create_client_id();

        (
            Self {
                screen: Screen::Connecting,
                i18n: I18n::load(&language),
                api_base_url: api_base_url.clone(),
                counter_id: counter_id.clone(),
                counter_password: counter_password.clone(),
                client_id: client_id.clone(),
                checkout_session: None,
                products: Vec::new(),
                product_images: HashMap::new(),
                search: String::new(),
                status: "Connecting to backend...".to_string(),
                loading_products: true,
                cart: Vec::new(),
                selected_product: None,
                quantity_input: String::new(),
                quantity_error: String::new(),
                measuring_weight: false,
                connection_failed: false,
                recovering_connection: false,
                manual_reconnect_available: false,
            },
            connect_task(
                api_base_url,
                counter_id,
                counter_password,
                client_id,
                Message::ConnectionFinished,
            ),
        )
    }
}

fn app_theme(_: &SelfCheckout) -> Theme {
    Theme::Dark
}

fn update(state: &mut SelfCheckout, message: Message) -> Task<Message> {
    match message {
        Message::StartPressed => {
            state.screen = Screen::Session;
            Task::none()
        }
        Message::RetryConnectionPressed => {
            state.connection_failed = false;
            state.manual_reconnect_available = false;
            state.status = "Retrying backend connection...".to_string();

            if state.recovering_connection {
                recovery_task(state)
            } else {
                state.screen = Screen::Connecting;
                connect_task(
                    state.api_base_url.clone(),
                    state.counter_id.clone(),
                    state.counter_password.clone(),
                    state.client_id.clone(),
                    Message::ConnectionFinished,
                )
            }
        }
        Message::ConnectionFinished(result) => match result {
            Ok((products, checkout_session)) => {
                state.loading_products = false;
                state.connection_failed = false;
                state.manual_reconnect_available = false;
                state.status = "Connected".to_string();
                state.products = products;
                state.cart = checkout_session.cart.clone();
                state.checkout_session = Some(checkout_session);
                state.screen = if state.cart.is_empty() {
                    Screen::Welcome
                } else {
                    Screen::Session
                };

                product_image_tasks(&state.products)
            }
            Err(error) => {
                state.loading_products = false;
                state.products.clear();
                state.cart.clear();
                state.checkout_session = None;
                state.connection_failed = true;
                state.manual_reconnect_available = true;
                state.screen = Screen::Connecting;
                state.status = error;
                Task::none()
            }
        },
        Message::RecoveryFinished(result) => match result {
            Ok((products, checkout_session)) => {
                state.products = products;
                state.cart = checkout_session.cart.clone();
                state.checkout_session = Some(checkout_session);
                state.recovering_connection = false;
                state.connection_failed = false;
                state.manual_reconnect_available = false;
                state.status = "Connection restored".to_string();
                product_image_tasks(&state.products)
            }
            Err(error) => {
                state.connection_failed = true;
                state.manual_reconnect_available = true;
                state.status = error;
                Task::none()
            }
        },
        Message::ProductSelected(product_id) => {
            if state.recovering_connection {
                return Task::none();
            }

            let Some(product) = state.products.iter().find(|product| product.id == product_id)
            else {
                return Task::none();
            };

            state.selected_product = Some(product.clone());
            state.quantity_input.clear();
            state.quantity_error.clear();

            if product.unit == "kg" {
                state.measuring_weight = true;
                Task::perform(measure_weight(), Message::WeightMeasured)
            } else {
                state.measuring_weight = false;
                Task::none()
            }
        }
        Message::SearchChanged(value) => {
            state.search = value;
            Task::none()
        }
        Message::QuantityChanged(value) => {
            state.quantity_input = value;
            state.quantity_error.clear();
            Task::none()
        }
        Message::KeypadPressed(value) => {
            state.quantity_input.push(value);
            state.quantity_error.clear();
            Task::none()
        }
        Message::KeypadClear => {
            state.quantity_input.clear();
            state.quantity_error.clear();
            Task::none()
        }
        Message::KeypadBackspace => {
            state.quantity_input.pop();
            state.quantity_error.clear();
            Task::none()
        }
        Message::ConfirmAddToCart => {
            if state.recovering_connection {
                return Task::none();
            }

            let Some(product) = state.selected_product.clone() else {
                return Task::none();
            };

            if state.measuring_weight {
                return Task::none();
            }

            let quantity = if product.unit == "kg" {
                match state.quantity_input.trim().parse::<f64>() {
                    Ok(value) if value > 0.0 => value,
                    _ => {
                        state.quantity_error = "Podaj poprawna wage".to_string();
                        return Task::none();
                    }
                }
            } else {
                match state.quantity_input.trim().parse::<u32>() {
                    Ok(value) if value > 0 => value as f64,
                    _ => {
                        state.quantity_error = "Podaj poprawna ilosc sztuk".to_string();
                        return Task::none();
                    }
                }
            };

            let mut next_cart = state.cart.clone();
            next_cart.push(CartItem::from_product(&product, quantity));

            state.selected_product = None;
            state.quantity_input.clear();
            state.quantity_error.clear();
            state.measuring_weight = false;
            state.status = "Syncing cart...".to_string();

            sync_cart_task(state, next_cart)
        }
        Message::CancelAddToCart => {
            state.selected_product = None;
            state.quantity_input.clear();
            state.quantity_error.clear();
            state.measuring_weight = false;
            Task::none()
        }
        Message::ProductImageLoaded { product_id, result } => {
            if let Ok(handle) = result {
                state.product_images.insert(product_id, handle);
            }
            Task::none()
        }
        Message::WeightMeasured(weight) => {
            state.measuring_weight = false;
            state.quantity_input = format!("{weight:.2}");
            state.quantity_error.clear();
            Task::none()
        }
        Message::CartSynced(result) => match result {
            Ok(checkout_session) => {
                state.cart = checkout_session.cart.clone();
                state.checkout_session = Some(checkout_session);
                state.status.clear();
                Task::none()
            }
            Err(error) => {
                state.status = format!("Cart sync failed: {error}");
                state.recovering_connection = true;
                state.connection_failed = false;
                state.manual_reconnect_available = false;
                recovery_task(state)
            }
        },
        Message::PayPressed => {
            if state.recovering_connection {
                return Task::none();
            }

            let Some(checkout_session) = state.checkout_session.clone() else {
                return Task::none();
            };

            if state.cart.is_empty() {
                state.status = "Cart is empty".to_string();
                return Task::none();
            }

            state.status = "Finishing payment...".to_string();
            connect_after_payment_task(
                state.api_base_url.clone(),
                state.counter_id.clone(),
                state.counter_password.clone(),
                state.client_id.clone(),
                checkout_session.id,
            )
        }
        Message::PaymentFinished(result) => match result {
            Ok((products, checkout_session)) => {
                state.products = products;
                state.checkout_session = Some(checkout_session);
                state.cart.clear();
                state.search.clear();
                state.selected_product = None;
                state.quantity_input.clear();
                state.quantity_error.clear();
                state.measuring_weight = false;
                state.recovering_connection = false;
                state.screen = Screen::Welcome;
                state.status = "Payment completed".to_string();
                product_image_tasks(&state.products)
            }
            Err(error) => {
                state.status = format!("Payment sync failed: {error}");
                state.recovering_connection = true;
                state.connection_failed = false;
                state.manual_reconnect_available = false;
                recovery_task(state)
            }
        },
    }
}

fn view(state: &SelfCheckout) -> Element<'_, Message> {
    match state.screen {
        Screen::Connecting => connection_view(state),
        Screen::Welcome => welcome_view(&state.i18n),
        Screen::Session => session_view(
            &state.i18n,
            &state.search,
            &state.products,
            &state.product_images,
            &state.cart,
            state.loading_products,
            state.selected_product.as_ref(),
            &state.quantity_input,
            &state.quantity_error,
            state.measuring_weight,
            !state.cart.is_empty(),
            state.recovering_connection,
            &state.status,
            state.manual_reconnect_available,
        ),
    }
}

fn connection_view(state: &SelfCheckout) -> Element<'_, Message> {
    let mut content = column![text(&state.status).size(28)]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    if state.connection_failed {
        content =
            content.push(button(text("Retry connection")).on_press(Message::RetryConnectionPressed));
    } else {
        content = content.push(text("Trying 3 times with 3-second intervals..."));
    }

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn product_image_tasks(products: &[Product]) -> Task<Message> {
    let tasks = products.iter().filter_map(|product| {
        product.image_url.clone().map(|image_url| {
            let product_id = product.id.clone();
            Task::perform(fetch_image(image_url), move |result| {
                Message::ProductImageLoaded { product_id, result }
            })
        })
    });

    Task::batch(tasks)
}

fn connect_task(
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
    message: fn(Result<(Vec<Product>, CheckoutSession), String>) -> Message,
) -> Task<Message> {
    Task::perform(
        connect_backend(api_base_url, counter_id, counter_password, client_id),
        message,
    )
}

fn connect_after_payment_task(
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
    session_id: String,
) -> Task<Message> {
    Task::perform(
        pay_and_reconnect(
            api_base_url,
            counter_id,
            counter_password,
            client_id,
            session_id,
        ),
        Message::PaymentFinished,
    )
}

fn sync_cart_task(state: &SelfCheckout, cart: Vec<CartItem>) -> Task<Message> {
    let Some(checkout_session) = state.checkout_session.clone() else {
        return Task::none();
    };

    Task::perform(
        sync_cart(
            state.api_base_url.clone(),
            state.counter_id.clone(),
            state.counter_password.clone(),
            state.client_id.clone(),
            checkout_session.id,
            cart,
        ),
        Message::CartSynced,
    )
}

fn recovery_task(state: &SelfCheckout) -> Task<Message> {
    Task::perform(
        connect_backend(
            state.api_base_url.clone(),
            state.counter_id.clone(),
            state.counter_password.clone(),
            state.client_id.clone(),
        ),
        Message::RecoveryFinished,
    )
}

async fn connect_backend(
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
) -> Result<(Vec<Product>, CheckoutSession), String> {
    if counter_id.is_empty() || counter_password.is_empty() {
        return Err("Missing CHECKOUT_COUNTER_ID or CHECKOUT_COUNTER_PASSWORD".to_string());
    }

    let mut last_error = "Backend is unavailable".to_string();

    for attempt in 1..=3 {
        match try_connect(&api_base_url, &counter_id, &counter_password, &client_id) {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = format!("Connection attempt {attempt}/3 failed: {error}");
                if attempt < 3 {
                    thread::sleep(Duration::from_secs(3));
                }
            }
        }
    }

    Err(last_error)
}

fn try_connect(
    api_base_url: &str,
    counter_id: &str,
    counter_password: &str,
    client_id: &str,
) -> Result<(Vec<Product>, CheckoutSession), String> {
    check_backend_health(api_base_url)?;
    let products = fetch_products_blocking(api_base_url)?;
    let checkout_session = connect_session(api_base_url, counter_id, counter_password, client_id)?;
    Ok((products, checkout_session))
}

fn check_backend_health(api_base_url: &str) -> Result<(), String> {
    let response = reqwest::blocking::get(health_url(api_base_url))
        .map_err(|error| format!("Failed to reach backend: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Backend health check failed: {error}"))?;

    response
        .json::<bool>()
        .map_err(|error| format!("Invalid backend health response: {error}"))?;

    Ok(())
}

fn connect_session(
    api_base_url: &str,
    counter_id: &str,
    counter_password: &str,
    client_id: &str,
) -> Result<CheckoutSession, String> {
    let client = reqwest::blocking::Client::new();
    let payload = ConnectPayload {
        counter_id: counter_id.to_string(),
        password: counter_password.to_string(),
        client_id: client_id.to_string(),
    };

    client
        .post(connect_session_url(api_base_url))
        .json(&payload)
        .send()
        .map_err(|error| format!("Failed to connect checkout session: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to connect checkout session: {error}"))?
        .json::<CheckoutSession>()
        .map_err(|error| format!("Failed to decode checkout session: {error}"))
}

async fn sync_cart(
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
    session_id: String,
    cart: Vec<CartItem>,
) -> Result<CheckoutSession, String> {
    let client = reqwest::blocking::Client::new();
    let payload = SyncCartPayload {
        counter_id,
        password: counter_password,
        client_id,
        cart,
    };

    client
        .put(session_cart_url(&api_base_url, &session_id))
        .json(&payload)
        .send()
        .map_err(|error| format!("Failed to sync cart: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to sync cart: {error}"))?
        .json::<CheckoutSession>()
        .map_err(|error| format!("Failed to decode checkout session: {error}"))
}

async fn pay_and_reconnect(
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
    session_id: String,
) -> Result<(Vec<Product>, CheckoutSession), String> {
    let client = reqwest::blocking::Client::new();
    let payload = ConnectPayload {
        counter_id: counter_id.clone(),
        password: counter_password.clone(),
        client_id: client_id.clone(),
    };

    client
        .post(session_pay_url(&api_base_url, &session_id))
        .json(&payload)
        .send()
        .map_err(|error| format!("Failed to finish payment: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to finish payment: {error}"))?;

    connect_backend(api_base_url, counter_id, counter_password, client_id).await
}

fn fetch_products_blocking(api_base_url: &str) -> Result<Vec<Product>, String> {
    reqwest::blocking::get(products_url(api_base_url))
        .map_err(|error| format!("Failed to fetch products: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to fetch products: {error}"))?
        .json::<ProductsResponse>()
        .map(|response| response.data)
        .map_err(|error| format!("Failed to decode products: {error}"))
}

async fn fetch_image(image_url: String) -> Result<ProductImage, String> {
    let response = reqwest::blocking::get(image_url)
        .map_err(|error| format!("Failed to fetch product image: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to fetch product image: {error}"))?;

    let bytes = response
        .bytes()
        .map_err(|error| format!("Failed to read product image: {error}"))?;

    Ok(image::Handle::from_bytes(bytes.to_vec()))
}

async fn measure_weight() -> f64 {
    thread::sleep(Duration::from_secs(3));

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_millis())
        .unwrap_or(0);

    2.0 + (millis % 3001) as f64 / 1000.0
}

fn api_v1_base(api_base_url: &str) -> String {
    let trimmed = api_base_url.trim_end_matches('/');
    if trimmed.ends_with("/api/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/api/v1")
    }
}

fn products_url(api_base_url: &str) -> String {
    format!("{}/products/", api_v1_base(api_base_url))
}

fn health_url(api_base_url: &str) -> String {
    format!("{}/utils/health-check/", api_v1_base(api_base_url))
}

fn connect_session_url(api_base_url: &str) -> String {
    format!("{}/checkout-sessions/connect", api_v1_base(api_base_url))
}

fn session_cart_url(api_base_url: &str, session_id: &str) -> String {
    format!(
        "{}/checkout-sessions/{session_id}/cart",
        api_v1_base(api_base_url)
    )
}

fn session_pay_url(api_base_url: &str, session_id: &str) -> String {
    format!("{}/checkout-sessions/{session_id}/pay", api_v1_base(api_base_url))
}

fn load_or_create_client_id() -> String {
    let path = client_id_path();
    if let Ok(value) = fs::read_to_string(&path) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let client_id = Uuid::new_v4().to_string();
    let _ = fs::write(path, &client_id);
    client_id
}

fn client_id_path() -> PathBuf {
    env::var("CLIENT_ID_STORAGE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".self-checkout-client-id"))
}
