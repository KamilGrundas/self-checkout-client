use crate::checkout::CartItem;
use crate::i18n::I18n;
use crate::message::Message;
use crate::product::{Category, Product, ProductImage};
use crate::ui::{
    action_button_style, floating_panel_style, primary_button_disabled_style,
    primary_button_selected_style, primary_button_style, scrollable_style,
};
use iced::widget::{
    button, column, container, image, opaque, row, scrollable, stack, text, text_input,
};
use iced::{Color, Element, Length};
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
    recovering_connection: bool,
    connection_status: &'a str,
    manual_reconnect_available: bool,
) -> Element<'a, Message> {
    let filtered_products: Vec<&Product> = products
        .iter()
        .filter(|product| {
            selected_category_key == "all" || product.category_key == selected_category_key
        })
        .collect();

    let category_buttons = category_filter_row(i18n, categories, selected_category_key);

    let mut products_list = column![].spacing(8).width(Length::Fill);

    for chunk in filtered_products.chunks(5) {
        let mut tiles_row = row![].spacing(8).width(Length::Fill);

        for product in chunk {
            let image_content: Element<'_, Message> =
                if let Some(handle) = product_images.get(&product.id) {
                    image(handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fixed(110.0))
                        .into()
                } else {
                    container(text("..."))
                        .width(Length::Fill)
                        .height(Length::Fixed(110.0))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .into()
                };

            let tile = button(
                container(column![image_content, text(&product.name),].spacing(8))
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fixed(220.0)),
            )
            .width(Length::FillPortion(1))
            .style(primary_button_style)
            .on_press(Message::ProductSelected(product.id.clone()));

            tiles_row = tiles_row.push(tile);
        }

        for _ in chunk.len()..5 {
            tiles_row = tiles_row.push(container(text("")).width(Length::FillPortion(1)));
        }

        products_list = products_list.push(tiles_row);
    }

    if loading_products {
        products_list = products_list.push(text(i18n.t("loading_products")));
    } else if filtered_products.is_empty() {
        products_list = products_list.push(text(i18n.t("no_products")));
    }

    let products_panel = container(
        column![
            category_buttons,
            container(
                scrollable(products_list)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new()
                            .width(12)
                            .margin(2)
                            .scroller_width(12)
                            .spacing(14),
                    ))
                    .style(scrollable_style)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(16)
    .style(floating_panel_style);

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

    let left_panel = column![
        container(products_panel).height(Length::Fill),
        container(utility_panel).height(Length::Fixed(132.0)),
    ]
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
    } else {
        base
    };

    if recovering_connection {
        connection_overlay(content, connection_status, manual_reconnect_available)
    } else {
        content
    }
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
) -> Element<'a, Message> {
    let mut buttons = row![].spacing(8).width(Length::Fill);

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
    button(text(label))
        .style(if key == selected_category_key {
            primary_button_selected_style
        } else {
            primary_button_style
        })
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
    let keypad = column![
        row![
            button("1")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('1'))
                .width(Length::Fill),
            button("2")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('2'))
                .width(Length::Fill),
            button("3")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('3'))
                .width(Length::Fill),
        ]
        .spacing(8),
        row![
            button("4")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('4'))
                .width(Length::Fill),
            button("5")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('5'))
                .width(Length::Fill),
            button("6")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('6'))
                .width(Length::Fill),
        ]
        .spacing(8),
        row![
            button("7")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('7'))
                .width(Length::Fill),
            button("8")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('8'))
                .width(Length::Fill),
            button("9")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('9'))
                .width(Length::Fill),
        ]
        .spacing(8),
        row![
            button("C")
                .style(primary_button_style)
                .on_press(Message::KeypadClear)
                .width(Length::Fill),
            button("0")
                .style(primary_button_style)
                .on_press(Message::KeypadPressed('0'))
                .width(Length::Fill),
            button("⌫")
                .style(primary_button_style)
                .on_press(Message::KeypadBackspace)
                .width(Length::Fill),
        ]
        .spacing(8)
    ]
    .spacing(8);

    let modal_body = if measuring_weight {
        column![
            text(format!("{} [{}]", product.name, product.unit)).size(28),
            text(i18n.t("measuring_weight")).size(24),
        ]
        .spacing(12)
    } else if product.unit == "kg" {
        column![
            text(format!("{} [{}]", product.name, product.unit)).size(28),
            text_input(i18n.t("quantity_weight").as_str(), quantity_input).size(28),
            row![
                button(text(i18n.t("add")))
                    .style(primary_button_style)
                    .on_press(Message::ConfirmAddToCart),
                button(text(i18n.t("cancel")))
                    .style(primary_button_style)
                    .on_press(Message::CancelAddToCart),
            ]
            .spacing(8),
        ]
        .spacing(12)
    } else {
        column![
            text(format!("{} [{}]", product.name, product.unit)).size(28),
            text_input(i18n.t("quantity_count").as_str(), quantity_input)
                .on_input(Message::QuantityChanged)
                .size(28),
            keypad,
            row![
                button(text(i18n.t("add")))
                    .style(primary_button_style)
                    .on_press(Message::ConfirmAddToCart),
                button(text(i18n.t("cancel")))
                    .style(primary_button_style)
                    .on_press(Message::CancelAddToCart),
            ]
            .spacing(8),
        ]
        .spacing(12)
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

    let modal = container(modal_content.padding(16))
        .width(Length::Fixed(420.0))
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
