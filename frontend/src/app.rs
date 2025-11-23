use crate::front_structs::State;
use crate::table::Table;
use futures::{SinkExt, StreamExt, channel::mpsc::UnboundedSender};
use gloo_net::websocket::{Message, futures::WebSocket};
use macros::string;
use std::sync::LazyLock;
use structs::{EventType, MessageBack, MessageText, Role, User};
use sycamore::prelude::*;
use sycamore::rt::console_error;
use wasm_bindgen_futures::spawn_local;
use web_sys::SubmitEvent;

pub static HOST: LazyLock<String> = LazyLock::new(|| std::env!("BACKEND").to_string());

#[component]
pub fn App() -> View {
    let users = create_signal(Vec::<User>::new());
    let this_user = create_signal(None::<User>);
    let state = create_signal(State::NotLogged);
    let ws_sender: Signal<Option<UnboundedSender<Message>>> =
        create_signal(None::<UnboundedSender<Message>>);
    let user_name = create_signal(String::new());
    let room = create_signal(String::new());

    let master_is_there = create_selector(move || {
        users
            .get_clone()
            .into_iter()
            .any(|user| user.role() == Role::Master)
    });
    let user_role = create_signal(match master_is_there.get() {
        true => string!("Voter"),
        false => string!("Master"),
    });
    create_memo(move || {
        match master_is_there.get() {
            true => user_role.set(string!("Voter")),
            false => user_role.set(string!("Master")),
        }
        console_log!("Messages: {:?}", users.get_clone());
    });
    create_effect(move || {
        if user_role.get_clone().eq("Master") {
            room.set(String::new());
        }
    });
    spawn_local({
        let users = users.clone();
        let ws_sender = ws_sender.clone();
        async move {
            let ws = WebSocket::open(&HOST).expect("no se pudo conectar al websocket");
            console_log!("Connected to WebSocket");
            let (mut write, mut read) = ws.split();
            let (tx, mut rx) = futures::channel::mpsc::unbounded();
            ws_sender.set(Some(tx));
            spawn_local({
                let users = users.clone();
                async move {
                    while let Some(msg) = read.next().await {
                        if let Ok(Message::Text(txt)) = msg {
                            match serde_json::from_str::<MessageBack>(&txt) {
                                Ok(message) => {
                                    console_log!("Message: {:?}", message);
                                    room.set(message.room);
                                    users.set(message.users);
                                    state.set(State::Logged);
                                }
                                Err(e) => console_error!("Error: {}", e),
                            }
                        }
                    }
                }
            });
            spawn_local(async move {
                while let Some(msg) = rx.next().await {
                    let _ = write.send(msg).await;
                }
            });
        }
    });
    view! {
        (match state.get_clone(){
            State::NotLogged => view!{
                form(on:submit=move |ev:SubmitEvent|{
                    ev.prevent_default();
                    console_log!("Submitted");
                    spawn_local(async move {
                        let send = ws_sender.split().0;
                        console_dbg!(&user_role);
                        let user = User::new(user_role.get_clone().into(),user_name.get_clone().as_str(), None, room.get_clone());
                        this_user.set(Some(user.clone()));
                        send.get_clone().unwrap().send(Message::Text(serde_json::to_string(&MessageText{ message_type: EventType::Start, user }).unwrap())).await.unwrap();
                    });

                }){
                    select(bind:value=user_role, disabled = master_is_there.get()){
                        (match master_is_there.get(){
                            true=>view!{
                                option(value="Voter", initial_selected = true){"Dev/QA/BA"}
                            },
                            false=>view!{
                                option(value="Master", initial_selected = true){"Scrum Master"}
                                option(value="Voter"){"Dev/QA/BA"}
                            },
                        })
                    }
                    (match user_role.get_clone().as_ref(){
                        "Voter" => view!{
                            input(r#type="text", placeholder="Room", bind:value=room){}
                        },
                        _ => view!{},
                    })
                    input(r#type="text", placeholder="Your Name", bind:value=user_name){}
                    input(r#type="submit"){"Submit"}
                }
            },
            State::Logged => view!{ Table(user = this_user.get_clone().unwrap(), users = users, ws_sender = ws_sender) }
        })
    }
}
