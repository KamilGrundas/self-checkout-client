use crate::checkout::CartItem;
use crate::i18n::I18n;
use crate::message::Message;
use crate::product::{Category, Product, ProductImage};
use crate::ui::{
    action_button_style, danger_button_style, floating_panel_style, keypad_button_style,
    primary_button_disabled_style, primary_button_selected_style, primary_button_style,
    product_tile_button_style, scrollable_style,
};
use iced::widget::{
    button, column, container, image, opaque, row, scrollable, stack, text, text_input,
};
use iced::{Color, Element, Length};
use iced_aw::Spinner;
use std::collections::HashMap;
use std::path::Path;

pub fn session_view<'a>(
    i18n: &'a I18n,
    categories: &'a [Category],
    selected_category_key: &'a str,
    products: &'a [Product],
    product_images: &'a HashMap<String, ProductImage>,
    cart: &'a [CartItem],
    loading_products: bool,
    selected_product: Option<&'a Product>,
    quantity_input: &'a str,
    quantity_error: &'a str,
    measuring_weight: bool,
    can_pay: bool,
    show_shelf_placement_prompt: bool,
    shelf_ready_enabled: bool,
    recovering_connection: bool,
    connection_status: &'a str,
    manual_reconnect_available: bool,
    product_search_open: bool,
    classifying: bool,
    suggested_product_ids: &'a [String],
    product_page: usize,
    show_payment_method_modal: bool,
    show_payment_processing_modal: bool,
) -> Element<'a, Message> {
    // Products grid panel — only built when search is open
    let products_grid_panel = {
        let filtered_products: Vec<&Product> = if selected_category_key == "suggested" {
            suggested_product_ids
                .iter()
                .filter_map(|id| products.iter().find(|p| p.id == *id))
                .collect()
        } else {
            products
                .iter()
                .filter(|product| {
                    selected_category_key == "all" || product.category_key == selected_category_key
                })
                .collect()
        };

        let category_buttons = category_filter_row(
            i18n,
            categories,
            selected_category_key,
            suggested_product_ids,
        );

        const PRODUCTS_PER_PAGE: usize = 15;
        const COLS: usize = 5;
        const ROWS: usize = 3;

        let total_pages = if filtered_products.is_empty() {
            1
        } else {
            (filtered_products.len() + PRODUCTS_PER_PAGE - 1) / PRODUCTS_PER_PAGE
        };
        let current_page = product_page.min(total_pages.saturating_sub(1));
        let page_start = current_page * PRODUCTS_PER_PAGE;
        let page_products = &filtered_products
            [page_start..filtered_products.len().min(page_start + PRODUCTS_PER_PAGE)];

        let mut products_list = column![]
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);

        for chunk in page_products.chunks(COLS) {
            let mut tiles_row = row![]
                .spacing(8)
                .width(Length::Fill)
                .height(Length::FillPortion(1));

            for product in chunk {
                let image_content: Element<'_, Message> =
                    if let Some(handle) = product_images.get(&product.id) {
                        image(handle.clone())
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    } else {
                        container(text("..."))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                            .into()
                    };

                let tile = button(
                    container(
                        column![image_content, text(&product.name)]
                            .spacing(8)
                            .height(Length::Fill),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(product_tile_button_style)
                .on_press(Message::ProductSelected(product.id.clone()));

                tiles_row = tiles_row.push(tile);
            }

            for _ in chunk.len()..COLS {
                tiles_row = tiles_row.push(
                    container(text(""))
                        .width(Length::FillPortion(1))
                        .height(Length::Fill),
                );
            }

            products_list = products_list.push(tiles_row);
        }

        let rendered_rows = page_products.chunks(COLS).count();
        for _ in rendered_rows..ROWS {
            let empty_row = (0..COLS).fold(
                row![]
                    .spacing(8)
                    .width(Length::Fill)
                    .height(Length::FillPortion(1)),
                |r, _| {
                    r.push(
                        container(text(""))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill),
                    )
                },
            );
            products_list = products_list.push(empty_row);
        }

        if loading_products {
            products_list = products_list.push(text(i18n.t("loading_products")));
        } else if filtered_products.is_empty() {
            products_list = products_list.push(text(i18n.t("no_products")));
        }

        let page_label = text(format!("{}/{}", current_page + 1, total_pages)).size(24);

        let prev_btn: Element<'_, Message> = if current_page > 0 {
            button(text(i18n.t("prev_page")).size(22))
                .style(primary_button_style)
                .padding([14, 28])
                .on_press(Message::ProductPageChanged(current_page - 1))
                .into()
        } else {
            button(text(i18n.t("prev_page")).size(22))
                .style(primary_button_disabled_style)
                .padding([14, 28])
                .into()
        };

        let next_btn: Element<'_, Message> = if current_page + 1 < total_pages {
            button(text(i18n.t("next_page")).size(22))
                .style(primary_button_style)
                .padding([14, 28])
                .on_press(Message::ProductPageChanged(current_page + 1))
                .into()
        } else {
            button(text(i18n.t("next_page")).size(22))
                .style(primary_button_disabled_style)
                .padding([14, 28])
                .into()
        };

        let pagination_row = container(
            row![
                prev_btn,
                container(page_label)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                next_btn,
            ]
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill),
        )
        .width(Length::Fill);

        container(
            column![
                category_buttons,
                container(products_list)
                    .width(Length::Fill)
                    .height(Length::Fill),
                pagination_row,
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .style(floating_panel_style)
    };

    let utility_panel = container(
        row![
            row![
                utility_button("assets/ui/help.png", i18n.t("help"), Message::HelpPressed),
                utility_button(
                    "assets/ui/language.png",
                    i18n.t("language"),
                    Message::LanguagePressed,
                )
            ]
            .spacing(12)
            .width(Length::Shrink),
            container(text("0,00 kg").size(40))
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .center_y(Length::Fill)
        ]
        .spacing(16)
        .align_y(iced::alignment::Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(16)
    .style(floating_panel_style);

    let left_panel = if product_search_open {
        column![
            container(products_grid_panel).height(Length::Fill),
            container(utility_panel).height(Length::Fixed(132.0)),
        ]
    } else {
        let prompt_panel = container(
            container(text(i18n.t("place_product_prompt")).size(24))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(32)
        .style(floating_panel_style);

        let search_panel: Element<'_, Message> = if classifying {
            container(text(i18n.t("classifying")).size(22))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(floating_panel_style)
                .into()
        } else {
            button(
                container(text(i18n.t("search_product")).size(26))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(primary_button_style)
            .on_press(Message::SearchProductPressed)
            .into()
        };

        column![
            container(prompt_panel).height(Length::Fill),
            container(search_panel).height(Length::Fixed(132.0)),
            container(utility_panel).height(Length::Fixed(132.0)),
        ]
    }
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(16);

    let cart_header = text(i18n.t("cart")).size(30);
    let mut cart_items = column![].spacing(8).width(Length::Fill);

    if cart.is_empty() {
        cart_items = cart_items.push(text(i18n.t("cart_empty")));
    } else {
        for item in cart {
            cart_items = cart_items.push(
                column![
                    text(format!("{} ({})", item.name, item.quantity_label)),
                    text(format!(
                        "{:.2}/{}  |  {:.2} x {:.2} = {:.2}",
                        item.price, item.unit, item.quantity, item.price, item.line_total
                    ))
                ]
                .spacing(2),
            );
        }
    }

    let total: f64 = cart.iter().map(|item| item.line_total).sum();
    let total_display = if total.abs() < 0.005 { 0.0 } else { total };

    let pay_button = {
        let button = button(
            container(text(i18n.t("pay")).size(28))
                .width(Length::Fill)
                .center_x(Length::Fill),
        )
        .padding([14, 18])
        .width(Length::Fill)
        .style(if can_pay {
            primary_button_style
        } else {
            primary_button_disabled_style
        });

        if can_pay {
            button.on_press(Message::PayPressed)
        } else {
            button
        }
    };

    let cart_footer = column![
        text(format!("{}: {:.2}", i18n.t("total"), total_display)).size(24),
        pay_button,
    ]
    .spacing(12);

    let right_panel = container(
        column![
            cart_header,
            container(
                scrollable(cart_items)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new()
                            .width(12)
                            .margin(2)
                            .scroller_width(12)
                            .spacing(10),
                    ))
                    .style(scrollable_style)
                    .height(Length::Fill)
                    .width(Length::Fill),
            )
            .width(Length::Fill),
            container(cart_footer).width(Length::Fill)
        ]
        .height(Length::Fill)
        .spacing(12),
    )
    .padding(16)
    .style(floating_panel_style);

    let content: Element<'_, Message> = column![
        row![
            container(left_panel).width(Length::FillPortion(3)),
            container(right_panel).width(Length::FillPortion(1))
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
    ]
    .height(Length::Fill)
    .into();

    let base: Element<'_, Message> = container(content)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    let content = if let Some(product) = selected_product {
        modal_view(
            i18n,
            base,
            product,
            quantity_input,
            quantity_error,
            measuring_weight,
        )
    } else if show_shelf_placement_prompt {
        ml_label_prompt_view(i18n, base, shelf_ready_enabled)
    } else if show_payment_method_modal {
        payment_method_view(i18n, base)
    } else if show_payment_processing_modal {
        payment_processing_view(i18n, base)
    } else {
        base
    };

    if recovering_connection {
        connection_overlay(content, connection_status, manual_reconnect_available)
    } else {
        content
    }
}

fn payment_method_view<'a>(i18n: &'a I18n, base: Element<'a, Message>) -> Element<'a, Message> {
    let dim = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            iced::widget::container::Style::default()
                .background(Color::from_rgba(0.0, 0.0, 0.0, 0.6))
        });

    let card_image: Element<'_, Message> = if Path::new("assets/ui/credit-card.png").exists() {
        image("assets/ui/credit-card.png")
            .width(Length::Fixed(120.0))
            .height(Length::Fixed(120.0))
            .into()
    } else {
        text("CARD").size(48).into()
    };

    let modal = container(
        column![
            text(i18n.t("payment_choose_method")).size(30),
            button(
                container(
                    column![card_image, text(i18n.t("payment_method_card")).size(24)]
                        .spacing(12)
                        .align_x(iced::alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            )
            .style(action_button_style)
            .padding([20, 24])
            .width(Length::Fill)
            .height(Length::Fixed(280.0))
            .on_press(Message::PaymentMethodSelected),
            button(
                container(text(i18n.t("payment_back")).size(24))
                    .width(Length::Fill)
                    .center_x(Length::Fill),
            )
            .style(danger_button_style)
            .padding([18, 24])
            .width(Length::Fill)
            .on_press(Message::PaymentMethodBackPressed),
        ]
        .spacing(20)
        .padding(24),
    )
    .width(Length::Fixed(560.0))
    .style(iced::widget::container::rounded_box);

    stack([
        base,
        stack([
            opaque(dim),
            container(modal)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn payment_processing_view<'a>(i18n: &'a I18n, base: Element<'a, Message>) -> Element<'a, Message> {
    let dim = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            iced::widget::container::Style::default()
                .background(Color::from_rgba(0.0, 0.0, 0.0, 0.6))
        });

    let modal = container(
        column![
            container(
                Spinner::new()
                    .width(Length::Fixed(80.0))
                    .height(Length::Fixed(80.0))
                    .circle_radius(6.0)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
            column![
                text(i18n.t("payment_follow_terminal")).size(28),
                text(i18n.t("payment_terminal_hint")).size(18),
            ]
            .spacing(6)
            .align_x(iced::alignment::Horizontal::Center),
        ]
        .spacing(12)
        .padding([56, 24])
        .height(Length::Fixed(340.0))
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .width(Length::Fixed(640.0))
    .style(iced::widget::container::rounded_box);

    stack([
        base,
        stack([
            opaque(dim),
            container(modal)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn ml_label_prompt_view<'a>(
    i18n: &'a I18n,
    base: Element<'a, Message>,
    shelf_ready_enabled: bool,
) -> Element<'a, Message> {
    let dim = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            iced::widget::container::Style::default()
                .background(Color::from_rgba(0.0, 0.0, 0.0, 0.6))
        });

    let ready_button = {
        let button = button(
            container(text(i18n.t("ready")).size(24))
                .width(Length::Fill)
                .center_x(Length::Fill),
        )
        .padding([18, 24])
        .width(Length::Fill)
        .style(if shelf_ready_enabled {
            primary_button_style
        } else {
            primary_button_disabled_style
        });

        if shelf_ready_enabled {
            button.on_press(Message::ShelfPlacementConfirmed)
        } else {
            button
        }
    };

    let modal = container(
        column![
            text(i18n.t("ml_place_product")).size(30),
            text(i18n.t("ml_ready_delay")).size(18),
            ready_button,
        ]
        .spacing(20)
        .padding(24),
    )
    .width(Length::Fixed(520.0))
    .style(iced::widget::container::rounded_box);

    stack([
        base,
        stack([
            opaque(dim),
            container(modal)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn utility_button<'a>(icon_path: &'a str, label: String, message: Message) -> Element<'a, Message> {
    let icon: Element<'a, Message> = if Path::new(icon_path).exists() {
        image(icon_path)
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(42.0))
            .into()
    } else {
        container(text(label.chars().next().unwrap_or('?').to_string()).size(24))
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(42.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    button(
        container(
            column![icon, text(label).size(16)]
                .spacing(8)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(Length::Fixed(120.0))
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .width(Length::Fixed(120.0))
    .height(Length::Fill)
    .style(action_button_style)
    .on_press(message)
    .into()
}

fn category_filter_row<'a>(
    i18n: &'a I18n,
    categories: &'a [Category],
    selected_category_key: &'a str,
    suggested_product_ids: &'a [String],
) -> Element<'a, Message> {
    let mut buttons = row![].spacing(8).width(Length::Fill);

    if !suggested_product_ids.is_empty() {
        buttons = buttons.push(category_button(
            i18n.t("category_suggested"),
            "suggested",
            selected_category_key,
        ));
    }

    buttons = buttons.push(category_button(i18n.t("all"), "all", selected_category_key));

    for category in categories.iter().filter(|category| category.key != "other") {
        buttons = buttons.push(category_button(
            category.name.clone(),
            &category.key,
            selected_category_key,
        ));
    }

    buttons = buttons.push(category_button(
        i18n.t("category_other"),
        "other",
        selected_category_key,
    ));

    scrollable(buttons)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new()
                .width(10)
                .margin(2)
                .scroller_width(10)
                .spacing(10),
        ))
        .style(scrollable_style)
        .height(Length::Shrink)
        .into()
}

fn category_button<'a>(
    label: String,
    key: &str,
    selected_category_key: &str,
) -> Element<'a, Message> {
    button(text(label).size(20))
        .style(if key == selected_category_key {
            primary_button_selected_style
        } else {
            primary_button_style
        })
        .padding([14, 22])
        .on_press(Message::CategorySelected(key.to_string()))
        .into()
}

fn modal_view<'a>(
    i18n: &'a I18n,
    base: Element<'a, Message>,
    product: &'a Product,
    quantity_input: &'a str,
    quantity_error: &'a str,
    measuring_weight: bool,
) -> Element<'a, Message> {
    let unit_label = localized_unit(i18n, &product.unit);

    let keypad = column![
        row![
            keypad_button("1", Message::KeypadPressed('1')).width(Length::Fill),
            keypad_button("2", Message::KeypadPressed('2')).width(Length::Fill),
            keypad_button("3", Message::KeypadPressed('3')).width(Length::Fill),
        ]
        .spacing(8),
        row![
            keypad_button("4", Message::KeypadPressed('4')).width(Length::Fill),
            keypad_button("5", Message::KeypadPressed('5')).width(Length::Fill),
            keypad_button("6", Message::KeypadPressed('6')).width(Length::Fill),
        ]
        .spacing(8),
        row![
            keypad_button("7", Message::KeypadPressed('7')).width(Length::Fill),
            keypad_button("8", Message::KeypadPressed('8')).width(Length::Fill),
            keypad_button("9", Message::KeypadPressed('9')).width(Length::Fill),
        ]
        .spacing(8),
        row![
            keypad_button("C", Message::KeypadClear).width(Length::Fill),
            keypad_button("0", Message::KeypadPressed('0')).width(Length::Fill),
            keypad_button("⌫", Message::KeypadBackspace).width(Length::Fill),
        ]
        .spacing(8)
    ]
    .spacing(12);

    let modal_body = if measuring_weight {
        column![
            text(format!("{} [{}]", product.name, unit_label)).size(40),
            text(i18n.t("measuring_weight")).size(30),
        ]
        .spacing(20)
    } else if product.unit == "kg" {
        column![
            text(format!("{} [{}]", product.name, unit_label)).size(40),
            text_input(i18n.t("quantity_weight").as_str(), quantity_input)
                .size(34)
                .padding(18),
            row![
                button(
                    container(text(i18n.t("add")).size(24))
                        .width(Length::Fill)
                        .center_x(Length::Fill),
                )
                .style(primary_button_style)
                .padding([22, 24])
                .width(Length::Fill)
                .on_press(Message::ConfirmAddToCart),
                button(
                    container(text(i18n.t("cancel")).size(24))
                        .width(Length::Fill)
                        .center_x(Length::Fill),
                )
                .style(danger_button_style)
                .padding([22, 24])
                .width(Length::Fill)
                .on_press(Message::CancelAddToCart),
            ]
            .spacing(12),
        ]
        .spacing(20)
    } else {
        column![
            text(format!("{} [{}]", product.name, unit_label)).size(40),
            text_input(i18n.t("quantity_count").as_str(), quantity_input)
                .on_input(Message::QuantityChanged)
                .size(34)
                .padding(18),
            keypad,
            row![
                button(
                    container(text(i18n.t("add")).size(24))
                        .width(Length::Fill)
                        .center_x(Length::Fill),
                )
                .style(primary_button_style)
                .padding([22, 24])
                .width(Length::Fill)
                .on_press(Message::ConfirmAddToCart),
                button(
                    container(text(i18n.t("cancel")).size(24))
                        .width(Length::Fill)
                        .center_x(Length::Fill),
                )
                .style(danger_button_style)
                .padding([22, 24])
                .width(Length::Fill)
                .on_press(Message::CancelAddToCart),
            ]
            .spacing(12),
        ]
        .spacing(20)
    };

    let modal_content = if quantity_error.is_empty() || measuring_weight {
        modal_body
    } else {
        modal_body.push(text(quantity_error))
    };

    let dim = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            iced::widget::container::Style::default()
                .background(Color::from_rgba(0.0, 0.0, 0.0, 0.6))
        });

    let modal = container(modal_content.padding(24))
        .width(Length::Fixed(560.0))
        .style(iced::widget::container::rounded_box);

    stack([
        base,
        stack([
            opaque(dim),
            container(modal)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn keypad_button<'a>(label: &'a str, message: Message) -> iced::widget::Button<'a, Message> {
    button(
        container(text(label).size(32))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .height(Length::Fixed(88.0))
    .style(keypad_button_style)
    .on_press(message)
}

fn localized_unit(i18n: &I18n, unit: &str) -> String {
    match unit {
        "pcs" => i18n.t("unit_pcs"),
        "kg" => i18n.t("unit_kg"),
        _ => unit.to_string(),
    }
}

fn connection_overlay<'a>(
    base: Element<'a, Message>,
    status: &'a str,
    manual_reconnect_available: bool,
) -> Element<'a, Message> {
    let dim = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            iced::widget::container::Style::default()
                .background(Color::from_rgba(0.0, 0.0, 0.0, 0.7))
        });

    let mut modal_content = column![
        text("Backend connection lost").size(28),
        text(status),
        text("Trying 3 times with 3-second intervals..."),
    ]
    .spacing(12)
    .padding(16);

    if manual_reconnect_available {
        modal_content = modal_content.push(
            button(text("Retry connection"))
                .style(primary_button_style)
                .on_press(Message::RetryConnectionPressed),
        );
    }

    let modal = container(modal_content)
        .width(Length::Fixed(480.0))
        .style(iced::widget::container::rounded_box);

    stack([
        base,
        stack([
            opaque(dim),
            container(modal)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
