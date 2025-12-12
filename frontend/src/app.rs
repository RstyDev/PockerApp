use crate::front_structs::State;
use crate::libs::send_message;
use crate::table::Table;
use futures::{SinkExt, StreamExt, channel::mpsc::UnboundedSender};
use gloo_net::websocket::{Message, futures::WebSocket};
use gloo_timers::future::sleep;
use macros::string;
use std::sync::LazyLock;
use std::time::Duration;
use structs::{EventType, MessageBack, MessageText, Role, User};
use sycamore::prelude::*;
use sycamore::rt::console_error;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{SubmitEvent, js_sys::Date, window};

pub static HOST: LazyLock<String> = LazyLock::new(|| std::env!("BACKEND").to_string());
fn get_token() -> Result<Option<User>, JsValue> {
    if let Some(window) = window()
        && let Some(storage) = window.session_storage()?
        && let Some(data) = storage.get_item("poker_token")?
    {
        if let Some(value) = data.split("|").nth(0) {
            console_dbg!(value);
            Ok(Some(
                serde_json::from_str(value)
                    .map_err(|e| JsValue::from_str(e.to_string().as_str()))?,
            ))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
fn save_token(token: String) -> Result<(), JsValue> {
    if let Some(window) = window()
        && let Some(storage) = window.session_storage()?
    {
        let expiry = Date::now() + 7200_000.0; // 2 hours in ms
        let data = format!("{}|{}", token, expiry);
        storage.set_item("poker_token", &data)?
    }
    Ok(())
}
#[component]
pub fn App() -> View {
    let master_name = create_signal(String::new());
    let first_loaded = create_signal(false);
    let dev_name = create_signal(String::new());
    let room_code = create_signal(String::new());
    let master_disabled =
        create_selector(move || dev_name.with(|v| v.len()) > 0 || room_code.with(|v| v.len()) > 0);
    let dev_disabled = create_selector(move || master_name.with(|v| v.len()) > 0);
    let error = create_signal(string!(""));
    let show_error = create_signal(false);
    let this_user = create_signal(match get_token() {
        Ok(user) => user,
        Err(e) => {
            console_dbg!(&e);
            None
        }
    });
    create_effect(move || {
        let err = show_error.get();
        spawn_local(async move {
            if err {
                sleep(Duration::from_millis(2000)).await;
                show_error.set(false);
                // console_dbg!("Copied to false now");
            }
        });
    });
    let users = create_signal(Vec::<User>::new());
    let show = create_signal(false);
    let state = create_signal(State::NotLogged);
    let ws_sender: Signal<Option<UnboundedSender<Message>>> =
        create_signal(None::<UnboundedSender<Message>>);
    console_log!("-.-.-before effect {:#?}", this_user.get_clone_untracked());
    create_effect(move || {
        // !used_token.get() &&
        if first_loaded.get()
            && let Some(_) = ws_sender.get_clone_untracked()
        {
            console_log!("-.-.-effect {:#?}", this_user.get_clone_untracked());
            //if this_user.with(|u| u.is_some()) {
            spawn_local(async move {
                //while this_user.with(|u|u.is_none());
                while let Some(user) = this_user.get_clone_untracked()
                    && let Err(e) = send_message(
                        *ws_sender,
                        MessageText {
                            message_type: EventType::SetUser,
                            user: user.clone(),
                        },
                    )
                    .await
                {
                    console_log!("{}", e);
                    console_log!("Not logged yet");
                }
            });
            /*} else {
                console_log!("No user is set")
            }*/
        }
    });
    let user_name = create_signal(String::new());
    let room = create_signal(String::new());

    create_memo(move || console_log!("Status: {:#?}", state.get_clone()));
    create_memo(move || {
        users.track();
        if users.with(|u| u.len()) == 0 {
            //this_user.set_silent(None);
            show.set_silent(false);
            user_name.set_silent(String::new());
            room.set_silent(String::new());
            state.set(State::NotLogged);
        }
    });
    spawn_local({
        async move {
            let ws = match WebSocket::open(&HOST) {
                Ok(socket) => socket,
                Err(e) => panic!("{e}"),
            };
            console_log!("Connected to WebSocket");
            let (mut write, mut read) = ws.split();

            let (tx, mut rx) = futures::channel::mpsc::unbounded();

            ws_sender.set(Some(tx));
            first_loaded.set(true);
            spawn_local({
                let users = users;
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
                                    if let Some(user) = current_user.as_ref() {
                                        if let Err(e) =
                                            save_token(serde_json::to_string(user).unwrap())
                                        {
                                            console_log!("User not saved");
                                            console_dbg!(&e);
                                        } else {
                                            console_log!("Token saved");
                                            console_dbg!(&(get_token()));
                                        }
                                    }
                                    this_user.set(current_user);
                                    show.set(message.show);
                                    room.set(message.room);
                                    let size = message.users.len();
                                    users.set(message.users);
                                    state.set(if size > 0 {
                                        State::Logged
                                    } else {
                                        State::NotLogged
                                    });
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
                    form(id="login_form",on:submit=move |ev:SubmitEvent|{
                        ev.prevent_default();
                        console_log!("Submitted");
                        spawn_local(async move {
                            let send = ws_sender.split().0;
                            let user_option = match (master_name.get_clone().as_ref(),dev_name.get_clone().as_ref(),room_code.get_clone().as_ref()){
                                ("","","") => Err(string!("It is required to select either a Scrum Master name or a Developer name")),
                                ("",dev_name,"") if !dev_name.is_empty() => Err(string!("If you are a Developer, ask the Scrum Master for the room code")),
                                ("","",room) if !room.is_empty() => Err(string!("Name input is required")),
                                ("",dev_name,room) if !dev_name.is_empty() && room.len() > 35 => Ok(User::new(Role::Voter, dev_name, None, room)),
                                (master_name,"",room) if !master_name.is_empty() => Ok(User::new(Role::Master, master_name, None, room)),
                                (_,_,_) => Err(string!("Unexpected Error"))
                            };
                            this_user.set(user_option.clone().ok());
                            match user_option {
                                Ok(user) => if let Err(e) = send_message(send, MessageText{ message_type: EventType::SetUser, user }).await {
                                    console_dbg!(&e);
                                },
                                Err(err) => {
                                    error.set(err);
                                    show_error.set(true);
                                },
                            }
                        });

                    }){
                        span(id="you_are"){"You are"}
                        div(id="master_form"){
                            label(r#for="master_name"){"Scrum Master"}
                            input(id= "master_name",placeholder="Your name",bind:value=master_name,disabled=master_disabled.get()){}
                        }
                        span(id="or"){"or"}
                        div(id="dev_form"){
                            label(r#for="voter_name"){"Developer/QA/BA"}
                            input(id="voter_name",placeholder="Your name",bind:value=dev_name,disabled=dev_disabled.get()){}
                            input(class="room_code",bind:value=room_code,placeholder="Room code from Scrum Master",disabled=dev_disabled.get()){}
                        }
                        input(id="login_submit",r#type="submit"){"Submit"}
                    }
                    article(class=format!("error {}",show_error.get())){
                        p(){
                            (format!("🚫 {}",error.get_clone()))
                        }
                    }
                },
                State::Logged => view!{ Table(user = this_user.get_clone().unwrap_or_default(),show = show, users = users, ws_sender = ws_sender) }
            })
        }
    }
}
