use crate::camera::CameraOption;
use crate::i18n::I18n;
use crate::message::Message;
use crate::ui::{
    primary_button_disabled_style, primary_button_selected_style, primary_button_style,
};
use iced::widget::{button, column, container, pick_list, row, text};
use iced::{Element, Length};

pub fn mode_selection_view<'a>(
    i18n: &'a I18n,
    cameras: &'a [CameraOption],
    selected_camera: Option<&'a CameraOption>,
    pending_mode_selection: Option<crate::MlMode>,
    camera_error: &'a str,
) -> Element<'a, Message> {
    let show_camera_picker = matches!(
        pending_mode_selection,
        Some(crate::MlMode::Label | crate::MlMode::On)
    );

    let ml_off = button(text("ML_off").size(28))
        .style(primary_button_style)
        .padding([16, 28])
        .on_press(Message::ModeSelected(crate::MlMode::Off));

    let ml_label = button(text("ML_label").size(28))
        .padding([16, 28])
        .style(if pending_mode_selection == Some(crate::MlMode::Label) {
            primary_button_selected_style
        } else {
            primary_button_style
        })
        .on_press(Message::ModeSelected(crate::MlMode::Label));

    let ml_on = button(text("ML_on").size(28))
        .padding([16, 28])
        .style(primary_button_disabled_style);

    let mut content = column![
        text(i18n.t("choose_mode")).size(42),
        row![ml_off, ml_label, ml_on].spacing(16),
    ]
    .spacing(20)
    .width(Length::Shrink);

    if show_camera_picker {
        let camera_picker = pick_list(cameras, selected_camera, Message::CameraSelected)
            .placeholder(i18n.t("camera_select"))
            .width(Length::Fixed(360.0));

        content = content.push(camera_picker);
    }

    if !camera_error.is_empty() {
        content = content.push(text(camera_error).size(16));
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
