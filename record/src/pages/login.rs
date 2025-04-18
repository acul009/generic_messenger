use adaptors::{discord::Discord, Messanger};
use futures::join;
use iced::{
    alignment::Vertical::Bottom,
    widget::{column, combo_box::State, Button, Column, ComboBox, Container, TextInput},
    Alignment, Element, Task,
};
use smol::lock::RwLock;
use std::{error, fmt::Display, sync::Arc};
use strum::EnumString;

use crate::auth::AuthStore;

use super::{chat::MessangerWindow, MyAppMessage, Page, UpdateResult};

// TODO: Make adapters handle the functionality of this enum
#[derive(Debug, Clone, EnumString)]
pub enum Platform {
    Discord,
    Test,
}
impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Discord => f.write_str("Discord"),
            Platform::Test => f.write_str("Test"),
        }
    }
}
impl Platform {
    pub fn to_messanger(&self, auth: String) -> Arc<dyn Messanger> {
        match self {
            Self::Discord => Arc::new(Discord::new(&auth)),
            Self::Test => todo!(),
        }
    }
    fn get_login_methods(&self) -> Vec<LoginMethods> {
        match self {
            Platform::Discord => vec![LoginMethods::Token],
            Platform::Test => vec![LoginMethods::Unkown],
        }
    }
}
// ===

enum LoginMethods {
    Token,
    Unkown,
}

#[derive(Debug, Clone)]
pub enum Message {
    PlatformInput(Platform),
    TokenInput(String),
    SubmitToken,
    LoginSuccess(Arc<dyn Messanger>),
    LoginError(String),
}

pub enum Action {
    Login(Arc<dyn Messanger>),
    Run(Task<Message>),
    None,
}

pub struct Login {
    platform: State<Platform>,
    selected_platform: Platform,
    token: String,
    error: Option<String>,
}
impl Login {
    pub fn new() -> Self {
        // TODO: Automate addition of new enum varients in here
        let service = State::new(vec![Platform::Discord, Platform::Test]);
        Self {
            platform: service,
            selected_platform: Platform::Test,
            token: String::new(),
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::PlatformInput(platform) => {
                println!("{:?}", platform);
                self.selected_platform = platform;
            }
            Message::TokenInput(change) => {
                self.token = change;
            }
            Message::SubmitToken => {
                // TODO: Disable submit button until the operation ether

                let platform = self.selected_platform.clone();
                let token = self.token.clone();

                return Action::Run(Task::future(async move {
                    let messanger = platform.to_messanger(token);
                    // Todo: check if login works
                    if true {
                        Message::LoginSuccess(messanger)
                    } else {
                        Message::LoginError("Invalid token".to_string())
                    }
                }));
            }
            Message::LoginSuccess(messenger) => {
                return Action::Login(messenger);
            }
            Message::LoginError(error) => {
                self.error = Some(error);
            }
        }

        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let width = 360.0;

        let select_platform = ComboBox::new(
            &self.platform,
            "Platform",
            Some(&self.selected_platform),
            |platform| Message::PlatformInput(platform),
        );

        let auth_input = self
            .selected_platform
            .get_login_methods()
            .iter()
            .filter_map(|method| match method {
                LoginMethods::Token => Some(Element::from(
                    TextInput::new("Token", self.token.as_str())
                        .on_input(|text| Message::TokenInput(text)),
                )),
                LoginMethods::Unkown => None,
            })
            .fold(Column::new(), |column, widget| column.push(widget));

        let content = column![
            "Login",
            select_platform,
            auth_input,
            Button::new("Submit").on_press(Message::SubmitToken)
        ]
        .width(iced::Length::Fixed(width))
        .align_x(Alignment::Center)
        .spacing(20);

        Container::new(content)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}
