use crate::MlMode;
use crate::camera::CameraOption;
use crate::i18n::I18n;
use crate::message::Message;
use crate::ui::{
    action_button_style, primary_button_disabled_style, primary_button_selected_style,
    primary_button_style,
};
use iced::widget::{
    button, column, container, image, opaque, pick_list, row, scrollable, stack, text,
};
use iced::{Color, Element, Length};

#[allow(clippy::too_many_arguments)]
pub fn settings_overlay<'a>(
    i18n: &'a I18n,
    base: Element<'a, Message>,
    ml_mode: MlMode,
    cameras: &'a [CameraOption],
    selected_shelf_camera: Option<&'a CameraOption>,
    selected_scale_camera: Option<&'a CameraOption>,
    shelf_preview: Option<&'a image::Handle>,
    scale_preview: Option<&'a image::Handle>,
    shelf_camera_error: &'a str,
    scale_camera_error: &'a str,
) -> Element<'a, Message> {
    let has_camera = selected_shelf_camera.is_some() || selected_scale_camera.is_some();

    // Mode radio buttons
    let mode_off = mode_radio_button("ML off", MlMode::Off, ml_mode, true);
    let mode_on = mode_radio_button("ML on", MlMode::On, ml_mode, has_camera);
    let mode_label = mode_radio_button("ML label", MlMode::Label, ml_mode, has_camera);

    let mode_row = row![mode_off, mode_on, mode_label].spacing(12);

    // Camera pickers with clear buttons
    let shelf_picker = pick_list(cameras, selected_shelf_camera, Message::CameraSelected)
        .placeholder(i18n.t("camera_select"))
        .width(Length::Fill);

    let shelf_picker_row = if selected_shelf_camera.is_some() {
        row![
            shelf_picker,
            button(text("\u{2715}").size(18))
                .padding([8, 12])
                .style(action_button_style)
                .on_press(Message::ClearShelfCamera),
        ]
        .spacing(8)
        .into()
    } else {
        Element::from(shelf_picker)
    };

    let scale_picker = pick_list(cameras, selected_scale_camera, Message::ScaleCameraSelected)
        .placeholder(i18n.t("scale_camera_select"))
        .width(Length::Fill);

    let scale_picker_row = if selected_scale_camera.is_some() {
        row![
            scale_picker,
            button(text("\u{2715}").size(18))
                .padding([8, 12])
                .style(action_button_style)
                .on_press(Message::ClearScaleCamera),
        ]
        .spacing(8)
        .into()
    } else {
        Element::from(scale_picker)
    };

    // Shelf camera section
    let mut shelf_section = column![
        text(i18n.t("shelf_camera_label")).size(18),
        shelf_picker_row,
    ]
    .spacing(8);

    if let Some(preview) = shelf_preview {
        shelf_section = shelf_section.push(
            container(
                image(preview.clone())
                    .content_fit(iced::ContentFit::Contain)
                    .width(Length::Fill)
                    .height(Length::Fixed(200.0)),
            )
            .width(Length::Fill),
        );
    }

    if !shelf_camera_error.is_empty() {
        shelf_section = shelf_section.push(
            text(shelf_camera_error)
                .size(14)
                .color(Color::from_rgb(0.8, 0.2, 0.2)),
        );
    }

    // Scale camera section
    let mut scale_section = column![
        text(i18n.t("scale_camera_label")).size(18),
        scale_picker_row,
    ]
    .spacing(8);

    if let Some(preview) = scale_preview {
        scale_section = scale_section.push(
            container(
                image(preview.clone())
                    .content_fit(iced::ContentFit::Contain)
                    .width(Length::Fill)
                    .height(Length::Fixed(200.0)),
            )
            .width(Length::Fill),
        );
    }

    if !scale_camera_error.is_empty() {
        scale_section = scale_section.push(
            text(scale_camera_error)
                .size(14)
                .color(Color::from_rgb(0.8, 0.2, 0.2)),
        );
    }

    let mut content = column![
        text(i18n.t("settings_title")).size(32),
        text(i18n.t("settings_mode")).size(20),
        mode_row,
    ]
    .spacing(16)
    .width(Length::Fill);

    if !has_camera {
        content = content.push(
            text(i18n.t("mode_requires_camera"))
                .size(14)
                .color(Color::from_rgba(0.0, 0.0, 0.0, 0.5)),
        );
    }

    let close_button = button(
        container(text(i18n.t("settings_close")).size(22))
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .padding([14, 28])
    .width(Length::Fill)
    .style(primary_button_style)
    .on_press(Message::ToggleSettings);

    content = content
        .push(shelf_section)
        .push(scale_section)
        .push(close_button);

    let dim = container("")
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| {
            iced::widget::container::Style::default()
                .background(Color::from_rgba(0.0, 0.0, 0.0, 0.6))
        });

    let modal = container(
        scrollable(container(content.padding(28)).width(Length::Fill)).height(Length::Shrink),
    )
    .width(Length::Fixed(600.0))
    .max_height(750.0)
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

fn mode_radio_button<'a>(
    label: &str,
    value: MlMode,
    selected: MlMode,
    enabled: bool,
) -> Element<'a, Message> {
    let is_selected = value == selected;
    let prefix = if is_selected {
        "\u{25CF} "
    } else {
        "\u{25CB} "
    };

    let btn = button(text(format!("{prefix}{label}")).size(22))
        .padding([12, 20])
        .style(if !enabled {
            primary_button_disabled_style
        } else if is_selected {
            primary_button_selected_style
        } else {
            action_button_style
        });

    if enabled {
        btn.on_press(Message::SettingsModeSelected(value)).into()
    } else {
        btn.into()
    }
}
