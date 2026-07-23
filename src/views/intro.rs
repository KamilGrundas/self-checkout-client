use crate::i18n::I18n;
use crate::message::Message;
use crate::ui::primary_button_style;
use iced::widget::{button, container, text};
use iced::{Element, Length};

pub fn welcome_view(i18n: &I18n) -> Element<'_, Message> {
    let start_button = button(
        container(text(i18n.t("start")).size(54))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(primary_button_style)
    .on_press(Message::StartPressed);

    container(start_button)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
