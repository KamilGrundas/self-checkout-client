use crate::i18n::I18n;
use crate::message::Message;
use iced::widget::{button, column, container, text};
use iced::{Element, Fill};

pub fn actions_view(i18n: &I18n) -> Element<'_, Message> {
    let actions = column![
        text(i18n.t("search")).size(42),
        button(text(i18n.t("add")).size(28)).on_press(Message::PlaceholderPressed),
        button(text(i18n.t("cart")).size(28)).on_press(Message::PlaceholderPressed),
        button(text(i18n.t("total")).size(28)).on_press(Message::PlaceholderPressed),
        button(text(i18n.t("cancel")).size(28)).on_press(Message::PlaceholderPressed),
    ]
    .spacing(24)
    .max_width(520);

    container(actions)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .padding(32)
        .into()
}
