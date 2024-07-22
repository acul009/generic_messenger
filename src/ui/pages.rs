use gtk4::glib::MainContext;
use gtk4::{prelude::*, Button, Entry, Image, Label, Orientation, Stack};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::Write, rc::Rc};
use tokio::sync::oneshot;

use super::components::components::{user_button, ChatData, DiscordUser};

use crate::discord::rest_api::utils::{download_image, init_data};
use crate::discord::websocket;
use crate::{runtime, LoginInfo};

pub fn login_page(parent_stack: Rc<Stack>) {
    let login = gtk4::Box::new(Orientation::Vertical, 5);
    let token_entry = Entry::new();
    token_entry.set_placeholder_text(Some("Place token here."));
    login.append(&token_entry);

    let submit_token = Button::new();
    submit_token.set_label("Submit");
    login.append(&submit_token);
    parent_stack.add_child(&login);

    submit_token.connect_clicked(move |_| {
        let entered_token = String::from(token_entry.text());
        if entered_token.is_empty() {
            return;
        }

        let data = match init_data(&entered_token) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        };

        let mut data_file = File::create("./public/loginInfo").expect("creation failed");
        data_file
            .write_all(entered_token.as_bytes())
            .expect("Write Failed");

        let user = LoginInfo {
            discord_token: Some(entered_token),
        };

        chat_page(parent_stack.clone(), user, Some(data));
        parent_stack.set_visible_child_name("chats");
        parent_stack.remove(&login);
    });
}

pub struct MainPanel {
    stack: gtk4::Stack,

    friend_list: gtk4::Box,
    friends: HashMap<String, Button>,

    chat_data: ChatData,
}

impl MainPanel {
    fn new() -> Self {
        let stack = gtk4::Stack::new();

        let friend_list = gtk4::Box::new(Orientation::Vertical, 4);
        let chat = gtk4::Box::new(Orientation::Vertical, 4);

        let pfp = Image::new();
        let username = Label::new(None);
        let text_field = Entry::new();

        // Chat layout
        chat.append(&pfp);
        chat.append(&username);
        chat.append(&text_field);
        // ===

        stack.add_named(&friend_list, Some("friend_list"));
        stack.add_named(&chat, Some("chat"));

        Self {
            stack,
            friend_list,
            friends: HashMap::new(),
            chat_data: ChatData { username, pfp },
        }
    }

    fn push_friend(&mut self, user: DiscordUser) {
        let user_box = gtk4::Box::new(Orientation::Horizontal, 5);

        let user_id = user.id.to_owned();
        let username = Label::new(Some(&user.username));

        let pfp = Image::from_file(&user.pfp);

        user_box.append(&pfp);
        user_box.append(&username);

        let button = Button::new();

        button.connect_clicked({
            let chat_data = self.chat_data.clone();
            let temp = self.stack.clone();
            move |_| {
                chat_data.set_current_chat(&user);
                temp.set_visible_child_name("chat");
            }
        });
        button.set_child(Some(&user_box));

        self.friend_list.append(&button);
        self.friends.insert(user_id, button);
    }
}

pub fn chat_page(parent_stack: Rc<Stack>, token_data: LoginInfo, info: Option<serde_json::Value>) {
    runtime().spawn(async move {
        // websocket::websocket::websocket_init(&token_data.discord_token.unwrap()).await;
    });

    let info = match info {
        Some(i) => i,
        None => init_data(&token_data.discord_token.unwrap()).unwrap(),
    };
    let friend_list = info.as_array().unwrap();

    let sections = gtk4::Box::new(Orientation::Horizontal, 0);
    // === Main Panel ===
    let mut main_panel = MainPanel::new();

    for f in friend_list {
        let username = f
            .get("user")
            .unwrap()
            .get("username")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let id = f.get("id").unwrap().as_str().unwrap().to_string();

        let pfp_id = f
            .get("user")
            .unwrap()
            .get("avatar")
            .unwrap()
            .as_str()
            .unwrap();

        let url = format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png?size=80",
            id, pfp_id
        );
        let pfp = Path::new(&format!("public/Contacts/pfp/{}", pfp_id)).to_owned();

        if !pfp.exists() {
            let path = pfp.clone();
            runtime().spawn({
                async move {
                    download_image(url, &path).await.unwrap();
                }
            });
        }
        main_panel.push_friend(DiscordUser { username, pfp, id });
    }

    // === Sidebar ===
    let sidebar = gtk4::Box::new(Orientation::Vertical, 20);

    // open friend list
    {
        let menue = gtk4::Box::new(Orientation::Vertical, 5);
        let friends = Button::new();
        friends.set_label("Friends");
        friends.connect_clicked({
            let stack = main_panel.stack.clone();
            move |_| {
                stack.set_visible_child_name("friend_list");
            }
        });
        menue.append(&friends);
        sidebar.append(&menue);
    }
    // DM list
    {
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

        let contact_list = gtk4::Box::new(Orientation::Vertical, 5);
        scroll.set_child(Some(&contact_list));

        for _ in 0..20 {
            user_button(
                &contact_list,
                &main_panel.stack,
                &main_panel.chat_data,
                "userid".to_string(),
            );
        }
        sidebar.append(&scroll);
    }
    // ===
    sections.append(&sidebar);
    sections.append(&main_panel.stack);
    // sections.append(&selected_chat);

    parent_stack.add_named(&sections, Some("chats"));
}
