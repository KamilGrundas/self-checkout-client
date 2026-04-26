mod camera;
mod checkout;
mod i18n;
mod message;
mod product;
mod settings;
mod ui;
mod views;

use crate::camera::{CameraOption, CameraWorker, CapturedFrame, SharedCameraHandle, list_cameras};
use crate::checkout::{CartItem, CheckoutSession, ConnectPayload, SyncCartPayload};
use crate::i18n::I18n;
use crate::message::{Message, PreviewFrames};
use crate::product::{CategoriesResponse, Category, Product, ProductImage, ProductsResponse};
use crate::settings::PersistedSettings;
use crate::ui::primary_button_style;

use iced::widget::{button, column, container, image, text};
use iced::{Element, Length, Subscription, Task, Theme, application, keyboard, window};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use views::intro::welcome_view;
use views::session::session_view;
use views::settings::settings_overlay;

fn main() -> iced::Result {
    let _ = dotenvy::dotenv();

    let app_env = env::var("APP_ENV")
        .unwrap_or_else(|_| "dev".to_string())
        .to_lowercase();
    let fullscreen = matches!(app_env.as_str(), "prod" | "production");

    application(SelfCheckout::new, update, view)
        .window(window::Settings {
            fullscreen,
            ..window::Settings::default()
        })
        .theme(app_theme)
        .subscription(subscription)
        .run()
}

#[derive(Debug, Clone, Default)]
enum Screen {
    #[default]
    Connecting,
    Welcome,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlMode {
    Off,
    Label,
    On,
}

#[derive(Debug, Clone)]
struct PendingShelfCapture {
    session_id: String,
    capture_index: usize,
    product_id: Option<String>,
    product_name: Option<String>,
}

struct SelfCheckout {
    screen: Screen,
    current_language: String,
    i18n: I18n,
    api_base_url: String,
    ml_api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
    checkout_session: Option<CheckoutSession>,
    categories: Vec<Category>,
    products: Vec<Product>,
    product_images: HashMap<String, ProductImage>,
    selected_category_key: String,
    status: String,
    loading_products: bool,
    ml_mode: MlMode,
    show_settings: bool,
    shelf_preview_handle: Option<image::Handle>,
    scale_preview_handle: Option<image::Handle>,
    cart: Vec<CartItem>,
    selected_product: Option<Product>,
    quantity_input: String,
    quantity_error: String,
    measuring_weight: bool,
    connection_failed: bool,
    recovering_connection: bool,
    manual_reconnect_available: bool,
    cameras: Vec<CameraOption>,
    // Shelf camera
    selected_shelf_camera: Option<CameraOption>,
    shelf_camera_worker: Option<CameraWorker>,
    shelf_camera_error: String,
    pending_shelf_capture: Option<PendingShelfCapture>,
    shelf_ready_enabled: bool,
    last_shelf_snapshot_session_id: Option<String>,
    last_shelf_snapshot_capture_index: Option<usize>,
    // Scale camera
    selected_scale_camera: Option<CameraOption>,
    scale_camera_worker: Option<CameraWorker>,
    scale_camera_error: String,
    last_scale_snapshot_session_id: Option<String>,
    last_scale_snapshot_capture_index: Option<usize>,
    // Product search
    product_search_open: bool,
    classifying: bool,
    suggested_product_ids: Vec<String>,
    product_page: usize,
}

impl SelfCheckout {
    fn new() -> (Self, Task<Message>) {
        let _ = dotenvy::dotenv();

        let language = env::var("DEFAULT_LANG").unwrap_or_else(|_| "en".to_string());
        let api_base_url =
            env::var("API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let ml_api_base_url =
            env::var("ML_API_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8001".to_string());
        let counter_id = env::var("CHECKOUT_COUNTER_ID").unwrap_or_default();
        let counter_password = env::var("CHECKOUT_COUNTER_PASSWORD").unwrap_or_default();
        let client_id = load_or_create_client_id();
        let (cameras, camera_error) = match list_cameras() {
            Ok(cameras) => (cameras, String::new()),
            Err(error) => (Vec::new(), error),
        };

        let persisted = PersistedSettings::load();
        let selected_shelf_camera = persisted.find_shelf_camera(&cameras).cloned();
        let selected_scale_camera = persisted.find_scale_camera(&cameras).cloned();
        let has_camera = selected_shelf_camera.is_some() || selected_scale_camera.is_some();
        let ml_mode = match persisted.ml_mode_enum() {
            mode @ (MlMode::On | MlMode::Label) if has_camera => mode,
            MlMode::On | MlMode::Label => MlMode::Off,
            mode => mode,
        };

        (
            Self {
                screen: Screen::Connecting,
                current_language: language.clone(),
                i18n: I18n::load(&language),
                api_base_url: api_base_url.clone(),
                ml_api_base_url,
                counter_id: counter_id.clone(),
                counter_password: counter_password.clone(),
                client_id: client_id.clone(),
                checkout_session: None,
                categories: Vec::new(),
                products: Vec::new(),
                product_images: HashMap::new(),
                selected_category_key: "all".to_string(),
                status: "Connecting to backend...".to_string(),
                loading_products: true,
                ml_mode,
                show_settings: false,
                shelf_preview_handle: None,
                scale_preview_handle: None,
                cart: Vec::new(),
                selected_product: None,
                quantity_input: String::new(),
                quantity_error: String::new(),
                measuring_weight: false,
                connection_failed: false,
                recovering_connection: false,
                manual_reconnect_available: false,
                cameras,
                selected_shelf_camera,
                shelf_camera_worker: None,
                shelf_camera_error: camera_error,
                pending_shelf_capture: None,
                shelf_ready_enabled: false,
                last_shelf_snapshot_session_id: None,
                last_shelf_snapshot_capture_index: None,
                selected_scale_camera,
                scale_camera_worker: None,
                scale_camera_error: String::new(),
                last_scale_snapshot_session_id: None,
                last_scale_snapshot_capture_index: None,
                product_search_open: false,
                classifying: false,
                suggested_product_ids: Vec::new(),
                product_page: 0,
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
    Theme::CatppuccinLatte
}

fn update(state: &mut SelfCheckout, message: Message) -> Task<Message> {
    match message {
        Message::StartPressed => {
            state.screen = Screen::Session;
            maybe_label_baseline_shelf_snapshot_task(state)
        }
        Message::ToggleSettings => {
            if !matches!(state.screen, Screen::Welcome) {
                return Task::none();
            }

            state.show_settings = !state.show_settings;

            if state.show_settings {
                // Start camera workers (non-blocking)
                if let Some(shelf_camera) = state.selected_shelf_camera.clone() {
                    if state.shelf_camera_worker.is_none() {
                        state.shelf_camera_worker =
                            Some(CameraWorker::start_nonblocking(shelf_camera));
                        state.shelf_camera_error.clear();
                    }
                }
                if let Some(scale_camera) = state.selected_scale_camera.clone() {
                    if state.scale_camera_worker.is_none() {
                        state.scale_camera_worker =
                            Some(CameraWorker::start_nonblocking(scale_camera));
                        state.scale_camera_error.clear();
                    }
                }
                return camera_preview_task(state);
            } else {
                // Closing settings — stop workers if mode is Off
                if state.ml_mode == MlMode::Off {
                    state.shelf_camera_worker = None;
                    state.scale_camera_worker = None;
                }
                state.shelf_preview_handle = None;
                state.scale_preview_handle = None;
                save_current_settings(state);
            }
            Task::none()
        }
        Message::SettingsModeSelected(mode) => {
            let has_camera =
                state.selected_shelf_camera.is_some() || state.selected_scale_camera.is_some();
            if matches!(mode, MlMode::On | MlMode::Label) && !has_camera {
                return Task::none();
            }
            state.ml_mode = mode;
            save_current_settings(state);
            Task::none()
        }
        Message::CameraPreviewTick(frames) => {
            if !state.show_settings {
                return Task::none();
            }

            if let Some(result) = frames.shelf {
                match result {
                    Ok(handle) => {
                        state.shelf_preview_handle = Some(handle);
                        state.shelf_camera_error.clear();
                    }
                    Err(error) => {
                        state.shelf_camera_error = error;
                    }
                }
            }
            if let Some(result) = frames.scale {
                match result {
                    Ok(handle) => {
                        state.scale_preview_handle = Some(handle);
                        state.scale_camera_error.clear();
                    }
                    Err(error) => {
                        state.scale_camera_error = error;
                    }
                }
            }

            camera_preview_task(state)
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
        Message::HelpPressed => Task::none(),
        Message::LanguagePressed => {
            let next_language = if state.current_language == "pl" {
                "en".to_string()
            } else {
                "pl".to_string()
            };

            state.current_language = next_language.clone();
            state.i18n = I18n::load(&next_language);
            state.quantity_error.clear();

            Task::none()
        }
        Message::CameraSelected(camera) => {
            state.shelf_camera_error.clear();
            state.shelf_camera_worker = None;
            state.shelf_preview_handle = None;

            if state.show_settings || state.ml_mode != MlMode::Off {
                state.shelf_camera_worker = Some(CameraWorker::start_nonblocking(camera.clone()));
            }

            state.selected_shelf_camera = Some(camera);
            save_current_settings(state);
            Task::none()
        }
        Message::ClearShelfCamera => {
            state.shelf_camera_worker = None;
            state.selected_shelf_camera = None;
            state.shelf_camera_error.clear();
            state.shelf_preview_handle = None;
            if matches!(state.ml_mode, MlMode::On | MlMode::Label)
                && state.selected_scale_camera.is_none()
            {
                state.ml_mode = MlMode::Off;
            }
            save_current_settings(state);
            Task::none()
        }
        Message::ScaleCameraSelected(camera) => {
            state.scale_camera_error.clear();
            state.scale_camera_worker = None;
            state.scale_preview_handle = None;

            if state.show_settings || state.ml_mode != MlMode::Off {
                state.scale_camera_worker = Some(CameraWorker::start_nonblocking(camera.clone()));
            }

            state.selected_scale_camera = Some(camera);
            save_current_settings(state);
            Task::none()
        }
        Message::ClearScaleCamera => {
            state.scale_camera_worker = None;
            state.selected_scale_camera = None;
            state.scale_camera_error.clear();
            state.scale_preview_handle = None;
            if matches!(state.ml_mode, MlMode::On | MlMode::Label)
                && state.selected_shelf_camera.is_none()
            {
                state.ml_mode = MlMode::Off;
            }
            save_current_settings(state);
            Task::none()
        }
        Message::ConnectionFinished(result) => match result {
            Ok((products, categories, checkout_session)) => {
                state.loading_products = false;
                state.connection_failed = false;
                state.manual_reconnect_available = false;
                state.status = "Connected".to_string();
                state.categories = categories;
                state.products = products;
                state.cart = checkout_session.cart.clone();
                state.checkout_session = Some(checkout_session.clone());
                state.screen = if state.cart.is_empty() {
                    Screen::Welcome
                } else {
                    Screen::Session
                };

                // Start camera workers if mode requires them
                if state.ml_mode != MlMode::Off {
                    if let Some(shelf_camera) = state.selected_shelf_camera.clone() {
                        if state.shelf_camera_worker.is_none() {
                            state.shelf_camera_worker =
                                Some(CameraWorker::start_nonblocking(shelf_camera));
                        }
                    }
                    if let Some(scale_camera) = state.selected_scale_camera.clone() {
                        if state.scale_camera_worker.is_none() {
                            state.scale_camera_worker =
                                Some(CameraWorker::start_nonblocking(scale_camera));
                        }
                    }
                }

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
            Ok((products, categories, checkout_session)) => {
                state.categories = categories;
                state.products = products;
                state.cart = checkout_session.cart.clone();
                state.checkout_session = Some(checkout_session.clone());
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

            let Some(product) = state
                .products
                .iter()
                .find(|product| product.id == product_id)
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
        Message::CategorySelected(category_key) => {
            state.selected_category_key = category_key;
            state.product_page = 0;
            Task::none()
        }
        Message::ProductPageChanged(page) => {
            state.product_page = page;
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
                        state.quantity_error = state.i18n.t("quantity_invalid_weight");
                        return Task::none();
                    }
                }
            } else {
                match state.quantity_input.trim().parse::<u32>() {
                    Ok(value) if value > 0 => value as f64,
                    _ => {
                        state.quantity_error = state.i18n.t("quantity_invalid_count");
                        return Task::none();
                    }
                }
            };

            let capture_index = state.cart.len() + 1;
            let session_id = state.checkout_session.as_ref().map(|s| s.id.clone());

            let mut next_cart = state.cart.clone();
            next_cart.push(CartItem::from_product(&product, quantity));

            state.selected_product = None;
            state.quantity_input.clear();
            state.quantity_error.clear();
            state.measuring_weight = false;
            state.product_search_open = false;
            state.suggested_product_ids.clear();
            state.status = "Syncing cart...".to_string();

            let sync_task = sync_cart_task(state, next_cart);

            let scale_task = if state.ml_mode == MlMode::Label {
                if let Some(sid) = session_id {
                    scale_snapshot_task(
                        state,
                        sid,
                        capture_index,
                        Some(product.id.clone()),
                        Some(product.name.clone()),
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            };

            Task::batch([sync_task, scale_task])
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
        Message::ShelfSnapshotUploaded {
            session_id,
            capture_index,
            result,
        } => {
            match result {
                Ok(()) => {
                    let should_advance = match (
                        state.last_shelf_snapshot_session_id.as_deref(),
                        state.last_shelf_snapshot_capture_index,
                    ) {
                        (Some(current_session_id), Some(current_capture_index)) => {
                            current_session_id != session_id
                                || capture_index >= current_capture_index
                        }
                        _ => true,
                    };

                    if should_advance {
                        state.last_shelf_snapshot_session_id = Some(session_id);
                        state.last_shelf_snapshot_capture_index = Some(capture_index);
                    }
                }
                Err(error) => {
                    state.shelf_camera_error = error;
                }
            }

            Task::none()
        }
        Message::ScaleSnapshotUploaded {
            session_id,
            capture_index,
            result,
        } => {
            match result {
                Ok(()) => {
                    let should_advance = match (
                        state.last_scale_snapshot_session_id.as_deref(),
                        state.last_scale_snapshot_capture_index,
                    ) {
                        (Some(current_session_id), Some(current_capture_index)) => {
                            current_session_id != session_id
                                || capture_index >= current_capture_index
                        }
                        _ => true,
                    };

                    if should_advance {
                        state.last_scale_snapshot_session_id = Some(session_id);
                        state.last_scale_snapshot_capture_index = Some(capture_index);
                    }
                }
                Err(error) => {
                    state.scale_camera_error = error;
                }
            }

            Task::none()
        }
        Message::ShelfPlacementReady => {
            state.shelf_ready_enabled = true;
            Task::none()
        }
        Message::ShelfPlacementConfirmed => {
            let Some(pending_capture) = state.pending_shelf_capture.take() else {
                return Task::none();
            };

            state.shelf_ready_enabled = false;
            shelf_snapshot_task(
                state,
                pending_capture.session_id,
                pending_capture.capture_index,
                pending_capture.product_id,
                pending_capture.product_name,
            )
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
                state.checkout_session = Some(checkout_session.clone());
                state.status.clear();
                if state.ml_mode == MlMode::Label && state.selected_shelf_camera.is_some() {
                    let capture_index = checkout_session.cart.len();
                    if capture_index > 0 {
                        state.pending_shelf_capture = Some(PendingShelfCapture {
                            session_id: checkout_session.id.clone(),
                            capture_index,
                            product_id: checkout_session
                                .cart
                                .last()
                                .map(|item| item.product_id.clone()),
                            product_name: checkout_session
                                .cart
                                .last()
                                .map(|item| item.name.clone()),
                        });
                        state.shelf_ready_enabled = false;
                        return Task::perform(wait_for_shelf_placement(), |_| {
                            Message::ShelfPlacementReady
                        });
                    }
                }

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
            Ok((products, categories, checkout_session)) => {
                state.categories = categories;
                state.products = products;
                state.checkout_session = Some(checkout_session.clone());
                state.cart.clear();
                state.selected_category_key = "all".to_string();
                state.selected_product = None;
                state.quantity_input.clear();
                state.quantity_error.clear();
                state.measuring_weight = false;
                state.product_search_open = false;
                state.classifying = false;
                state.suggested_product_ids.clear();
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
        Message::SearchProductPressed => {
            state.suggested_product_ids.clear();
            if state.ml_mode == MlMode::On {
                if let Some(camera_worker) = state.scale_camera_worker.as_ref() {
                    state.classifying = true;
                    let handle = camera_worker.shared_handle();
                    return Task::perform(
                        async move { handle.capture_fresh() },
                        Message::ScaleCameraFrameReady,
                    );
                }
            }
            state.product_search_open = true;
            Task::none()
        }
        Message::ScaleCameraFrameReady(frame_result) => {
            let classify_task = Task::perform(
                classify_product(state.ml_api_base_url.clone(), frame_result.clone()),
                Message::ClassifyFinished,
            );

            let scale_task = if let Some(session) = state.checkout_session.as_ref() {
                let capture_index = state.cart.len() + 1;
                let already_sent = state.last_scale_snapshot_session_id.as_deref()
                    == Some(session.id.as_str())
                    && state
                        .last_scale_snapshot_capture_index
                        .is_some_and(|last| last >= capture_index);

                if !already_sent && !state.ml_api_base_url.trim().is_empty() {
                    let url = ml_scale_snapshots_url(&state.ml_api_base_url, &session.id);
                    let upload_session_id = session.id.clone();
                    Task::perform(
                        upload_snapshot(url, frame_result, capture_index, None, None),
                        move |result| Message::ScaleSnapshotUploaded {
                            session_id: upload_session_id,
                            capture_index,
                            result,
                        },
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            };

            Task::batch([classify_task, scale_task])
        }
        Message::ClassifyFinished(result) => {
            state.classifying = false;
            state.product_search_open = true;
            match result {
                Ok(name_scores) => {
                    // Match returned names to products (case-insensitive), preserve confidence order
                    state.suggested_product_ids = name_scores
                        .iter()
                        .filter_map(|(name, _score)| {
                            let name_lower = name.to_lowercase();
                            state
                                .products
                                .iter()
                                .find(|p| p.name.to_lowercase() == name_lower)
                                .map(|p| p.id.clone())
                        })
                        .collect();
                    if !state.suggested_product_ids.is_empty() {
                        state.selected_category_key = "suggested".to_string();
                    } else {
                        state.selected_category_key = "all".to_string();
                    }
                }
                Err(_) => {
                    state.selected_category_key = "all".to_string();
                }
            }
            Task::none()
        }
    }
}

fn subscription(state: &SelfCheckout) -> Subscription<Message> {
    if matches!(state.screen, Screen::Welcome) && !state.show_settings {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(ref c),
                ..
            } if c.as_str() == "o" => Some(Message::ToggleSettings),
            _ => None,
        })
    } else {
        Subscription::none()
    }
}

fn view(state: &SelfCheckout) -> Element<'_, Message> {
    match state.screen {
        Screen::Connecting => connection_view(state),
        Screen::Welcome => {
            let base = welcome_view(&state.i18n);
            if state.show_settings {
                settings_overlay(
                    &state.i18n,
                    base,
                    state.ml_mode,
                    &state.cameras,
                    state.selected_shelf_camera.as_ref(),
                    state.selected_scale_camera.as_ref(),
                    state.shelf_preview_handle.as_ref(),
                    state.scale_preview_handle.as_ref(),
                    &state.shelf_camera_error,
                    &state.scale_camera_error,
                )
            } else {
                base
            }
        }
        Screen::Session => session_view(
            &state.i18n,
            &state.categories,
            &state.selected_category_key,
            &state.products,
            &state.product_images,
            &state.cart,
            state.loading_products,
            state.selected_product.as_ref(),
            &state.quantity_input,
            &state.quantity_error,
            state.measuring_weight,
            !state.cart.is_empty(),
            state.pending_shelf_capture.is_some(),
            state.shelf_ready_enabled,
            state.recovering_connection,
            &state.status,
            state.manual_reconnect_available,
            state.product_search_open,
            state.classifying,
            &state.suggested_product_ids,
            state.product_page,
        ),
    }
}

fn connection_view(state: &SelfCheckout) -> Element<'_, Message> {
    let mut content = column![text(&state.status).size(28)]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    if state.connection_failed {
        content = content.push(
            button(text("Retry connection"))
                .style(primary_button_style)
                .on_press(Message::RetryConnectionPressed),
        );
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
    message: fn(Result<(Vec<Product>, Vec<Category>, CheckoutSession), String>) -> Message,
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

fn maybe_label_baseline_shelf_snapshot_task(state: &SelfCheckout) -> Task<Message> {
    if state.ml_mode != MlMode::Label {
        return Task::none();
    }

    let Some(checkout_session) = state.checkout_session.as_ref() else {
        return Task::none();
    };

    if !checkout_session.cart.is_empty() {
        return Task::none();
    }

    shelf_snapshot_task(state, checkout_session.id.clone(), 0, None, None)
}

fn shelf_snapshot_task(
    state: &SelfCheckout,
    session_id: String,
    capture_index: usize,
    product_id: Option<String>,
    product_name: Option<String>,
) -> Task<Message> {
    let Some(camera_worker) = state.shelf_camera_worker.as_ref() else {
        return Task::none();
    };

    if state.ml_api_base_url.trim().is_empty() {
        return Task::none();
    }

    if state.last_shelf_snapshot_session_id.as_deref() == Some(session_id.as_str())
        && state
            .last_shelf_snapshot_capture_index
            .is_some_and(|last| last >= capture_index)
    {
        return Task::none();
    }

    let snapshot_result = camera_worker.capture_now();
    let upload_session_id = session_id.clone();
    Task::perform(
        upload_snapshot(
            ml_shelf_snapshots_url(&state.ml_api_base_url, &session_id),
            snapshot_result,
            capture_index,
            product_id,
            product_name,
        ),
        move |result| Message::ShelfSnapshotUploaded {
            session_id: upload_session_id,
            capture_index,
            result,
        },
    )
}

fn scale_snapshot_task(
    state: &SelfCheckout,
    session_id: String,
    capture_index: usize,
    product_id: Option<String>,
    product_name: Option<String>,
) -> Task<Message> {
    let Some(camera_worker) = state.scale_camera_worker.as_ref() else {
        return Task::none();
    };

    if state.ml_api_base_url.trim().is_empty() {
        return Task::none();
    }

    if state.last_scale_snapshot_session_id.as_deref() == Some(session_id.as_str())
        && state
            .last_scale_snapshot_capture_index
            .is_some_and(|last| last >= capture_index)
    {
        return Task::none();
    }

    let snapshot_result = camera_worker.capture_now();
    let upload_session_id = session_id.clone();
    Task::perform(
        upload_snapshot(
            ml_scale_snapshots_url(&state.ml_api_base_url, &session_id),
            snapshot_result,
            capture_index,
            product_id,
            product_name,
        ),
        move |result| Message::ScaleSnapshotUploaded {
            session_id: upload_session_id,
            capture_index,
            result,
        },
    )
}

async fn connect_backend(
    api_base_url: String,
    counter_id: String,
    counter_password: String,
    client_id: String,
) -> Result<(Vec<Product>, Vec<Category>, CheckoutSession), String> {
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
) -> Result<(Vec<Product>, Vec<Category>, CheckoutSession), String> {
    check_backend_health(api_base_url)?;
    let products = fetch_products_blocking(api_base_url)?;
    let categories = fetch_categories_blocking(api_base_url)?;
    let checkout_session = connect_session(api_base_url, counter_id, counter_password, client_id)?;
    Ok((products, categories, checkout_session))
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
) -> Result<(Vec<Product>, Vec<Category>, CheckoutSession), String> {
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

fn fetch_categories_blocking(api_base_url: &str) -> Result<Vec<Category>, String> {
    reqwest::blocking::get(categories_url(api_base_url))
        .map_err(|error| format!("Failed to fetch categories: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to fetch categories: {error}"))?
        .json::<CategoriesResponse>()
        .map(|response| response.data)
        .map_err(|error| format!("Failed to decode categories: {error}"))
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

async fn upload_snapshot(
    url: String,
    snapshot_result: Result<CapturedFrame, String>,
    capture_index: usize,
    product_id: Option<String>,
    product_name: Option<String>,
) -> Result<(), String> {
    let frame = snapshot_result?;
    let file_part = reqwest::blocking::multipart::Part::bytes(frame.bytes)
        .file_name(frame.file_name.to_string())
        .mime_str(frame.content_type)
        .map_err(|error| format!("Failed to prepare snapshot upload: {error}"))?;

    let mut form = reqwest::blocking::multipart::Form::new()
        .text("capture_index", capture_index.to_string())
        .part("file", file_part);

    if let Some(product_id) = product_id {
        form = form.text("product_id", product_id);
    }

    if let Some(product_name) = product_name {
        form = form.text("product_name", product_name);
    }

    reqwest::blocking::Client::new()
        .post(url)
        .multipart(form)
        .send()
        .map_err(|error| format!("Failed to upload snapshot: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Failed to upload snapshot: {error}"))?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct PredictionPublic {
    scores: HashMap<String, f64>,
}

async fn classify_product(
    ml_api_base_url: String,
    snapshot_result: Result<CapturedFrame, String>,
) -> Result<Vec<(String, f64)>, String> {
    let frame = snapshot_result.map_err(|e| format!("Camera capture failed: {e}"))?;
    let file_part = reqwest::blocking::multipart::Part::bytes(frame.bytes)
        .file_name(frame.file_name.to_string())
        .mime_str(frame.content_type)
        .map_err(|error| format!("Failed to prepare classify request: {error}"))?;

    let form = reqwest::blocking::multipart::Form::new().part("file", file_part);

    let prediction = reqwest::blocking::Client::new()
        .post(ml_classify_url(&ml_api_base_url))
        .multipart(form)
        .send()
        .map_err(|error| format!("Failed to call classify: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Classify endpoint error: {error}"))?
        .json::<PredictionPublic>()
        .map_err(|error| format!("Failed to decode classify response: {error}"))?;

    let mut scored: Vec<(String, f64)> = prediction.scores.into_iter().collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(scored.into_iter().take(10).collect())
}

async fn measure_weight() -> f64 {
    thread::sleep(Duration::from_secs(3));

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_millis())
        .unwrap_or(0);

    2.0 + (millis % 3001) as f64 / 1000.0
}

async fn wait_for_shelf_placement() {
    thread::sleep(Duration::from_secs(3));
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

fn categories_url(api_base_url: &str) -> String {
    format!("{}/categories/", api_v1_base(api_base_url))
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
    format!(
        "{}/checkout-sessions/{session_id}/pay",
        api_v1_base(api_base_url)
    )
}

fn ml_api_v1_base(api_base_url: &str) -> String {
    let trimmed = api_base_url.trim_end_matches('/');
    if trimmed.ends_with("/api/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/api/v1")
    }
}

fn ml_shelf_snapshots_url(api_base_url: &str, session_id: &str) -> String {
    format!(
        "{}/checkout-sessions/{session_id}/shelf-snapshots",
        ml_api_v1_base(api_base_url)
    )
}

fn ml_classify_url(api_base_url: &str) -> String {
    format!("{}/inference/classify", ml_api_v1_base(api_base_url))
}

fn ml_scale_snapshots_url(api_base_url: &str, session_id: &str) -> String {
    format!(
        "{}/checkout-sessions/{session_id}/scale-snapshots",
        ml_api_v1_base(api_base_url)
    )
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

fn camera_preview_task(state: &SelfCheckout) -> Task<Message> {
    let shelf_handle = state
        .shelf_camera_worker
        .as_ref()
        .map(|w| w.shared_handle());
    let scale_handle = state
        .scale_camera_worker
        .as_ref()
        .map(|w| w.shared_handle());

    Task::perform(
        fetch_preview_frames(shelf_handle, scale_handle),
        Message::CameraPreviewTick,
    )
}

async fn fetch_preview_frames(
    shelf: Option<SharedCameraHandle>,
    scale: Option<SharedCameraHandle>,
) -> PreviewFrames {
    thread::sleep(Duration::from_millis(200));

    PreviewFrames {
        shelf: shelf.map(|h| {
            let frame = h.latest_frame()?;
            decode_preview_handle(&frame.bytes)
        }),
        scale: scale.map(|h| {
            let frame = h.latest_frame()?;
            decode_preview_handle(&frame.bytes)
        }),
    }
}

/// Decode image bytes, resize to small preview, and return as RGBA handle.
fn decode_preview_handle(bytes: &[u8]) -> Result<image::Handle, String> {
    let img = ::image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode camera frame: {e}"))?;

    // Resize to preview dimensions to minimize GPU texture churn
    let preview = img.thumbnail(320, 240);
    let rgba = preview.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Ok(image::Handle::from_rgba(w, h, rgba.into_raw()))
}

fn save_current_settings(state: &SelfCheckout) {
    PersistedSettings::from_state(
        state.ml_mode,
        state.selected_shelf_camera.as_ref(),
        state.selected_scale_camera.as_ref(),
    )
    .save();
}
