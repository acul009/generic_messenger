use std::sync::Arc;

use auth::AuthStore;
use iced::{window, Element, Task};
use pages::{chat::MessangerWindow, Login, MyAppMessage};
use smol::lock::RwLock;

mod auth;
mod pages;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting");
    iced::daemon(App::title(), App::update, App::view)
        .run_with(|| {
            let app = App::default();
            let (_window_id, window_task) = window::open(window::Settings::default());

            (app, window_task.then(|_| Task::none()))
        })
        .inspect_err(|err| println!("{}", err))?;

    Ok(())
}

enum Page {
    Login(Login),
    Todo,
}

struct App {
    auth_store: AuthStore,
    page: Page,
}

impl Default for App {
    fn default() -> Self {
        let auth_store = AuthStore::new("./LoginInfo".into());

        let page = if auth_store.is_empty() {
            Page::Login(Login::new())
        } else {
            Page::Todo
        };

        Self { auth_store, page }
    }
}

impl App {
    fn title() -> &'static str {
        "record"
    }
    fn update(&mut self, message: MyAppMessage) -> impl Into<Task<MyAppMessage>> {
        match message {
            MyAppMessage::Login(message) => {
                if let Page::Login(login) = &mut self.page {
                    let action = login.update(message);

                    match action {
                        pages::login::Action::None => Task::none(),
                        pages::login::Action::Login(messenger) => {
                            self.auth_store.add_auth(messenger);
                            self.page = Page::Todo;
                            Task::none()
                        }
                        pages::login::Action::Run(task) => task.map(MyAppMessage::Login),
                    }
                } else {
                    Task::none()
                }
            }
            MyAppMessage::Chat(message) => {
                //TODO
                Task::none()
            }
        }
    }
    fn view(&self, _window: window::Id) -> Element<MyAppMessage> {
        match &self.page {
            Page::Login(login) => login.view().map(MyAppMessage::Login),
            Page::Todo => iced::widget::text("Todo").into(),
        }
    }
}
