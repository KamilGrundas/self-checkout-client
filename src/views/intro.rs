use crate::i18n::I18n;
use crate::message::Message;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, text};
use iced::{Element, Length};

pub fn welcome_view(i18n: &I18n) -> Element<'_, Message> {
    let start_button = button(
        text(i18n.t("start"))
            .size(54)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .on_press(Message::StartPressed);

    container(start_button)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}
