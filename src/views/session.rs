use crate::checkout::CartItem;
use crate::i18n::I18n;
use crate::message::Message;
use crate::product::{Product, ProductImage};
use iced::widget::{
    button, column, container, image, opaque, row, scrollable, stack, text, text_input,
};
use iced::{Color, Element, Length};
use std::collections::HashMap;

pub fn session_view<'a>(
    i18n: &'a I18n,
    search: &'a str,
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
    let query = search.to_lowercase();
    let filtered_products: Vec<&Product> = products
        .iter()
        .filter(|product| product.name.to_lowercase().contains(&query))
        .collect();

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
                container(
                    column![
                        image_content,
                        text(&product.name),
                        text(&product.unit),
                        text(format!("{:.2}", product.price)),
                    ]
                    .spacing(8),
                )
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fixed(220.0)),
            )
            .width(Length::FillPortion(1))
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

    let left_panel = column![
        text_input(i18n.t("search").as_str(), search).on_input(Message::SearchChanged),
        scrollable(products_list).width(Length::Fill).height(Length::Fill)
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .spacing(10)
    .padding(10);

    let mut cart_list = column![text(i18n.t("cart"))].spacing(8);

    if cart.is_empty() {
        cart_list = cart_list.push(text(i18n.t("cart_empty")));
    } else {
        for item in cart {
            cart_list = cart_list.push(
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
    cart_list = cart_list.push(text(format!("{}: {:.2}", i18n.t("total"), total)).size(24));
    if can_pay {
        cart_list = cart_list.push(button(text(i18n.t("pay"))).on_press(Message::PayPressed));
    }

    let right_panel = container(scrollable(cart_list).height(Length::Fill)).padding(10);

    let content: Element<'_, Message> = column![row![
        container(left_panel).width(Length::FillPortion(3)),
        container(right_panel).width(Length::FillPortion(1))
    ]
    .width(Length::Fill)
    .height(Length::Fill)]
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
            button("1").on_press(Message::KeypadPressed('1')).width(Length::Fill),
            button("2").on_press(Message::KeypadPressed('2')).width(Length::Fill),
            button("3").on_press(Message::KeypadPressed('3')).width(Length::Fill),
        ]
        .spacing(8),
        row![
            button("4").on_press(Message::KeypadPressed('4')).width(Length::Fill),
            button("5").on_press(Message::KeypadPressed('5')).width(Length::Fill),
            button("6").on_press(Message::KeypadPressed('6')).width(Length::Fill),
        ]
        .spacing(8),
        row![
            button("7").on_press(Message::KeypadPressed('7')).width(Length::Fill),
            button("8").on_press(Message::KeypadPressed('8')).width(Length::Fill),
            button("9").on_press(Message::KeypadPressed('9')).width(Length::Fill),
        ]
        .spacing(8),
        row![
            button("C").on_press(Message::KeypadClear).width(Length::Fill),
            button("0").on_press(Message::KeypadPressed('0')).width(Length::Fill),
            button("⌫")
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
                button(text(i18n.t("add"))).on_press(Message::ConfirmAddToCart),
                button(text(i18n.t("cancel"))).on_press(Message::CancelAddToCart),
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
                button(text(i18n.t("add"))).on_press(Message::ConfirmAddToCart),
                button(text(i18n.t("cancel"))).on_press(Message::CancelAddToCart),
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
        modal_content =
            modal_content.push(button(text("Retry connection")).on_press(Message::RetryConnectionPressed));
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
