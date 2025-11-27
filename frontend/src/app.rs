use crate::front_structs::State;
use crate::table::Table;
use futures::{SinkExt, StreamExt, channel::mpsc::UnboundedSender};
use gloo_net::websocket::{Message, futures::WebSocket};
use macros::string;
use std::sync::LazyLock;
use structs::{EventType, MessageBack, MessageText, User};
use sycamore::prelude::*;
use sycamore::rt::console_error;
use wasm_bindgen_futures::spawn_local;
use web_sys::SubmitEvent;

pub static HOST: LazyLock<String> = LazyLock::new(|| std::env!("BACKEND").to_string());

#[component]
pub fn App() -> View {
    let users = create_signal(Vec::<User>::new());
    let this_user = create_signal(None::<User>);
    let show = create_signal(false);
    let state = create_signal(State::NotLogged);
    let ws_sender: Signal<Option<UnboundedSender<Message>>> =
        create_signal(None::<UnboundedSender<Message>>);
    let user_name = create_signal(String::new());
    let room = create_signal(String::new());

    let user_role = create_signal(string!("Master"));
    create_memo(move || {
        if users.with(|u| u.len()) == 0 {
            this_user.set_silent(None);
            show.set_silent(false);
            user_name.set_silent(String::new());
            room.set_silent(String::new());
            state.set(State::NotLogged);
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
                                    let mut current_user = this_user.get_clone();
                                    let room_id = message.room.clone();
                                    current_user = current_user.as_mut().map(|user| {
                                        user.set_room(room_id);
                                        user.to_owned()
                                    });
                                    this_user.set(current_user);
                                    show.set(message.show);
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
        section(id="board"){
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
                            send.get_clone().unwrap().send(Message::Text(serde_json::to_string(&MessageText{ message_type: EventType::SetUser, user }).unwrap())).await.unwrap();
                        });

                    }){
                        select(id="select_role",bind:value=user_role){
                            option(value="Master", initial_selected = true){"Scrum Master"}
                            option(value="Voter"){"Dev/QA/BA"}
                        }
                        (match user_role.get_clone().as_ref(){
                            "Voter" => view!{
                                input(class="room_code",r#type="text", placeholder="Room", bind:value=room){}
                            },
                            _ => view!{},
                        })
                        input(r#type="text", placeholder="Your Name", bind:value=user_name){}
                        input(r#type="submit"){"Submit"}
                    }
                },
                State::Logged => view!{ Table(user = this_user.get_clone().unwrap_or_default(),show = show, users = users, ws_sender = ws_sender) }
            })
        }
    }
}
