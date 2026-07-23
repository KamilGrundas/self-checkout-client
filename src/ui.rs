use iced::Background;
use iced::Color;
use iced::Shadow;
use iced::Theme;
use iced::Vector;
use iced::border;
use iced::widget::button;
use iced::widget::container;
use iced::widget::scrollable;

const PRIMARY_BUTTON_COLOR: Color = Color {
    r: 0x20 as f32 / 255.0,
    g: 0x6F as f32 / 255.0,
    b: 0x5B as f32 / 255.0,
    a: 1.0,
};

const PRIMARY_BUTTON_HOVER_COLOR: Color = Color {
    r: 0x1B as f32 / 255.0,
    g: 0x61 as f32 / 255.0,
    b: 0x4F as f32 / 255.0,
    a: 1.0,
};

const PRIMARY_BUTTON_SELECTED_COLOR: Color = Color {
    r: 0x17 as f32 / 255.0,
    g: 0x52 as f32 / 255.0,
    b: 0x43 as f32 / 255.0,
    a: 1.0,
};

const PRIMARY_BUTTON_DISABLED_COLOR: Color = Color {
    r: 0x9A as f32 / 255.0,
    g: 0xB8 as f32 / 255.0,
    b: 0xB0 as f32 / 255.0,
    a: 1.0,
};

const ACTION_BUTTON_BG_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

const ACTION_BUTTON_HOVER_BG_COLOR: Color = Color {
    r: 0xF4 as f32 / 255.0,
    g: 0xF8 as f32 / 255.0,
    b: 0xF6 as f32 / 255.0,
    a: 1.0,
};

const DANGER_BUTTON_COLOR: Color = Color {
    r: 0xC9 as f32 / 255.0,
    g: 0x41 as f32 / 255.0,
    b: 0x41 as f32 / 255.0,
    a: 1.0,
};

const DANGER_BUTTON_HOVER_COLOR: Color = Color {
    r: 0xAF as f32 / 255.0,
    g: 0x38 as f32 / 255.0,
    b: 0x38 as f32 / 255.0,
    a: 1.0,
};

const KEYPAD_BUTTON_BORDER_COLOR: Color = Color {
    r: 0xD8 as f32 / 255.0,
    g: 0xE4 as f32 / 255.0,
    b: 0xDF as f32 / 255.0,
    a: 1.0,
};

const SCROLLBAR_BG_COLOR: Color = Color {
    r: 0xDE as f32 / 255.0,
    g: 0xE9 as f32 / 255.0,
    b: 0xE4 as f32 / 255.0,
    a: 1.0,
};

const SCROLLBAR_SCROLLER_COLOR: Color = Color {
    r: 0x20 as f32 / 255.0,
    g: 0x6F as f32 / 255.0,
    b: 0x5B as f32 / 255.0,
    a: 0.9,
};

const SCROLLBAR_SCROLLER_HOVER_COLOR: Color = Color {
    r: 0x17 as f32 / 255.0,
    g: 0x52 as f32 / 255.0,
    b: 0x43 as f32 / 255.0,
    a: 1.0,
};

const PANEL_BACKGROUND_COLOR: Color = Color {
    r: 0xFB as f32 / 255.0,
    g: 0xFD as f32 / 255.0,
    b: 0xFC as f32 / 255.0,
    a: 1.0,
};

const PANEL_SHADOW_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.12,
};

const PRODUCT_TILE_SHADOW_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.05,
};

pub fn primary_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => PRIMARY_BUTTON_HOVER_COLOR,
        button::Status::Pressed => PRIMARY_BUTTON_HOVER_COLOR,
        _ => PRIMARY_BUTTON_COLOR,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: border::rounded(12),
        ..button::Style::default()
    }
}

pub fn primary_button_selected_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => PRIMARY_BUTTON_HOVER_COLOR,
        button::Status::Pressed => PRIMARY_BUTTON_HOVER_COLOR,
        _ => PRIMARY_BUTTON_SELECTED_COLOR,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: border::rounded(12),
        ..button::Style::default()
    }
}

pub fn primary_button_disabled_style(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(PRIMARY_BUTTON_DISABLED_COLOR)),
        text_color: Color::WHITE.scale_alpha(0.9),
        border: border::rounded(12),
        ..button::Style::default()
    }
}

pub fn action_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => ACTION_BUTTON_HOVER_BG_COLOR,
        button::Status::Pressed => ACTION_BUTTON_HOVER_BG_COLOR,
        _ => ACTION_BUTTON_BG_COLOR,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: PRIMARY_BUTTON_COLOR,
        border: border::rounded(16)
            .width(1)
            .color(Color::from_rgba(0.0, 0.0, 0.0, 0.06)),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 14.0,
        },
        ..button::Style::default()
    }
}

pub fn danger_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => DANGER_BUTTON_HOVER_COLOR,
        button::Status::Pressed => DANGER_BUTTON_HOVER_COLOR,
        _ => DANGER_BUTTON_COLOR,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: border::rounded(12),
        ..button::Style::default()
    }
}

pub fn product_tile_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgb8(0xFA, 0xFC, 0xFB),
        button::Status::Pressed => Color::from_rgb8(0xF6, 0xF9, 0xF7),
        _ => Color::WHITE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: PRIMARY_BUTTON_SELECTED_COLOR,
        border: border::rounded(18),
        shadow: Shadow {
            color: PRODUCT_TILE_SHADOW_COLOR,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 10.0,
        },
        ..button::Style::default()
    }
}

pub fn keypad_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => ACTION_BUTTON_HOVER_BG_COLOR,
        button::Status::Pressed => ACTION_BUTTON_HOVER_BG_COLOR,
        _ => ACTION_BUTTON_BG_COLOR,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: PRIMARY_BUTTON_SELECTED_COLOR,
        border: border::rounded(18)
            .width(1)
            .color(KEYPAD_BUTTON_BORDER_COLOR),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 10.0,
        },
        ..button::Style::default()
    }
}

pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let active_rail = scrollable::Rail {
        background: Some(Background::Color(SCROLLBAR_BG_COLOR)),
        border: border::rounded(999),
        scroller: scrollable::Scroller {
            background: Background::Color(SCROLLBAR_SCROLLER_COLOR),
            border: border::rounded(999),
        },
    };

    let highlighted_rail = scrollable::Rail {
        scroller: scrollable::Scroller {
            background: Background::Color(SCROLLBAR_SCROLLER_HOVER_COLOR),
            border: border::rounded(999),
        },
        ..active_rail
    };

    let style = match status {
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: if is_vertical_scrollbar_hovered {
                highlighted_rail
            } else {
                active_rail
            },
            horizontal_rail: if is_horizontal_scrollbar_hovered {
                highlighted_rail
            } else {
                active_rail
            },
            gap: Some(Background::Color(Color::TRANSPARENT)),
            auto_scroll: scrollable::default(theme, status).auto_scroll,
        },
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: if is_vertical_scrollbar_dragged {
                highlighted_rail
            } else {
                active_rail
            },
            horizontal_rail: if is_horizontal_scrollbar_dragged {
                highlighted_rail
            } else {
                active_rail
            },
            gap: Some(Background::Color(Color::TRANSPARENT)),
            auto_scroll: scrollable::default(theme, status).auto_scroll,
        },
        _ => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: active_rail,
            horizontal_rail: active_rail,
            gap: Some(Background::Color(Color::TRANSPARENT)),
            auto_scroll: scrollable::default(theme, status).auto_scroll,
        },
    };

    style
}

pub fn floating_panel_style(_: &Theme) -> container::Style {
    container::Style::default()
        .background(Background::Color(PANEL_BACKGROUND_COLOR))
        .border(border::rounded(20))
        .shadow(Shadow {
            color: PANEL_SHADOW_COLOR,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 24.0,
        })
}
