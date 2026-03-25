mod i18n;
mod message;
mod screen;
mod views;
use crate::i18n::I18n;

use crate::message::Message;
use crate::screen::Screen;

use iced::{Element, Task, Theme, application};
use std::env;

use views::actions::actions_view;
use views::intro::welcome_view;

fn main() -> iced::Result {
    application(SelfCheckout::new, update, view)
        .theme(app_theme)
        .run()
}

struct SelfCheckout {
    screen: Screen,
    i18n: I18n,
}

impl SelfCheckout {
    fn new() -> (Self, Task<Message>) {
        let _ = dotenvy::dotenv();

        let language = env::var("DEFAULT_LANG").unwrap_or_else(|_| "en".to_string());

        (
            Self {
                screen: Screen::Welcome,
                i18n: I18n::load(&language),
            },
            Task::none(),
        )
    }
}

fn app_theme(_: &SelfCheckout) -> Theme {
    Theme::TokyoNight
}

fn update(state: &mut SelfCheckout, message: Message) -> Task<Message> {
    match message {
        Message::StartPressed => {
            state.screen = Screen::Actions;
        }
        Message::PlaceholderPressed => {}
    }

    Task::none()
}

fn view(state: &SelfCheckout) -> Element<'_, Message> {
    match state.screen {
        Screen::Welcome => welcome_view(&state.i18n),
        Screen::Actions => actions_view(&state.i18n),
    }
}
