use crate::{libs::copy_to_clipboard, user_cards::UserCards};
use futures::{SinkExt, channel::mpsc::UnboundedSender};
use gloo_net::websocket::Message;
use std::rc::Rc;
use structs::{EventType, MessageText, Role, User};
use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::SubmitEvent;

#[component(inline_props)]
pub fn Table(
    user: User,
    users: Signal<Vec<User>>,
    show: Signal<bool>,
    ws_sender: Signal<Option<UnboundedSender<Message>>>,
) -> View {
    let is_master = user.role() == Role::Master;
    let split_users = create_selector(move || {
        let users = users
            .get_clone()
            .into_iter()
            .filter(|u| u.role() == Role::Voter)
            .collect::<Vec<User>>();
        let size = users.len();
        let (left, right) = users.split_at(size / 2);
        (left.to_vec(), right.to_vec())
    });
    let empty_room =
        create_selector(move || split_users.with(|(a, b)| a.is_empty() && b.is_empty()));
    let value: ReadSignal<f32> = create_selector(move || {
        let size = users.with(|u| u.len() as f32) - 1.0;
        let sum = users
            .get_clone()
            .into_iter()
            .filter_map(|u| (u.role() == Role::Voter).then_some(u.value()).flatten())
            .sum::<u8>() as f32;
        sum / size
    });
    let master = users
        .get_clone()
        .into_iter()
        .find(|user| user.role() == Role::Master);
    let number = create_signal(String::new());
    let code = Rc::new(
        master
            .as_ref()
            .map(|m| m.room().to_string())
            .unwrap_or_default(),
    );
    let code2 = code.clone();
    console_dbg!(&master);
    console_dbg!(&users);
    // let view = users.get_clone().into_iter().map(|user|{view!{article(){(user.value().unwrap_or_default().to_string())}}}).collect::<Vec<View>>();
    // let cards = users.get_clone().into_iter().map(|user| view!{article(class="card"){(user.value().unwrap_or_default().to_string())}}).collect<Vec<View>>();
    view! {
        (match is_master{
            true => view!{
                section(id="code_section"){
                    label(r#for="code"){"Connection Code"}
                    input(name="code",class="room_code",disabled=true,value=code2.to_string()){}
                    button(r#type="button",on:click = move |_| {
                        let code = code.clone();
                        spawn_local(async move {
                            match copy_to_clipboard(code.as_str()).await {
                                Ok(_) => console_log!("✅ Copiado al portapapeles"),
                                Err(e) => console_log!("❌ Error copiando: {:?}", e),
                            }

                        });
                    }){"Copy"}
                }
            },
            false => view!{},
        })
        main(){
            aside(id="left"){
                UserCards(users = split_users.get_clone().0.to_owned(), show = show)
            }
            (match empty_room.get() {
                false => {
                    let send = ws_sender.clone();
                    let user = user.clone();
                    view!{
                        section(id="center"){
                            (match is_master{
                                true => view!{
                                    button(on:click = move |_|{
                                        let send = send.to_owned();
                                        let user = user.to_owned();
                                        console_dbg!(&user);
                                        spawn_local(async move {
                                            send.get_clone().unwrap().send(Message::Text(serde_json::to_string(&MessageText{ message_type: EventType::Show, user }).unwrap())).await.unwrap();
                                        });
                                    }){"Show cards"}
                                },
                                false => view!{
                                    form(on:submit = move |ev:SubmitEvent| {
                                        ev.prevent_default();
                                        let ws_sender = ws_sender.clone();
                                        let mut user = user.clone();
                                        spawn_local(async move {
                                            let send = ws_sender.split().0;
                                            user.set_value(number.get_clone().parse().ok());
                                            send.get_clone().unwrap().send(Message::Text(serde_json::to_string(&MessageText{ message_type: EventType::SetUser, user }).unwrap())).await.unwrap();
                                        });
                                    }){
                                        input(r#type="number",bind:value=number){}
                                        input(r#type="submit"){"Vote"}
                                    }
                                },
                            })
                            div(class="card master") {
                                p(){
                                    (show.get().then(||value.get().to_string()).unwrap_or_default())
                                }
                            }
                        }
                    }
                },
                true => view!{},
            })
            aside(id="right"){
                UserCards(users = split_users.get_clone().1.to_owned(), show = show)
            }
        }
    }
}
