use crate::{
    libs::copy_to_clipboard,
    user_cards::{Side, UserCards},
};
use futures::{SinkExt, channel::mpsc::UnboundedSender};
use gloo_net::websocket::Message;
use gloo_timers::future::sleep;
use std::{collections::HashMap, rc::Rc, time::Duration};
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
    let copied = create_signal(false);
    create_effect(move || {
        let cop = copied.get();
        spawn_local(async move {
            if cop {
                sleep(Duration::from_millis(500)).await;
                copied.set(false);
                // console_dbg!("Copied to false now");
            }
        });
    });
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
    let value: ReadSignal<u8> = create_selector(move || {
        let numbers = users
            .get_clone()
            .into_iter()
            .filter_map(|u| u.value())
            .collect::<Vec<_>>();
        let mut map: HashMap<u8, u8> = HashMap::new();
        for num in numbers {
            let current = map.get(&num);
            map.insert(num, current.cloned().unwrap_or(0) + 1);
        }
        let mut max_k = 0;
        let mut max_v = 0;
        for (k, v) in map.into_iter() {
            if v > max_v {
                max_v = v;
                max_k = k
            }
        }
        max_k
    });
    let master = users
        .get_clone()
        .into_iter()
        .find(|user| user.role() == Role::Master);
    let master_name = Rc::new(
        master
            .as_ref()
            .map(|m| m.name().to_owned())
            .unwrap_or_default(),
    );
    let number = create_signal(String::new());
    let code = Rc::new(
        master
            .as_ref()
            .map(|m| m.room().to_owned())
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
                    article(id="code_article"){
                        label(r#for="code"){"Connection Code"}
                        input(name="code",class="room_code",disabled=true,value=code2.to_string()){}
                        button(r#type="button",on:click = move |_| {
                            if !copied.get(){
                                let code = code.clone();
                                spawn_local(async move {
                                    match copy_to_clipboard(&code).await {
                                        Ok(_) => {
                                            console_log!("✅ Copied to clipboard");
                                            copied.set(true);
                                        },
                                        Err(e) => console_log!("❌ Error copiando: {:?}", e),
                                    }
                                });
                            }
                        }){"Copy"}
                    }
                    article(class = match copied.get(){
                        true => "copied",
                        false => "not-copied",
                    }){
                        p(){"✅ Copied to clipboard"}
                    }
                }
            },
            false => view!{},
        })
        main(){

            (match empty_room.get() {
                false => {
                    let master_name = master_name.clone();
                    let send = ws_sender.clone();
                    let user = user.clone();
                    view!{
                        section(id="scrum_master_name"){
                            p(){ (format!("Scrum Master: {}",master_name)) }
                        }
                        aside(id="left"){
                            UserCards(users = split_users.get_clone().0.to_owned(), show = show, side = Side::Left)
                        }
                        section(id="center"){
                            (match is_master{
                                true => view!{
                                    (match show.get(){
                                        true => {
                                            let send = send.to_owned();
                                            let user = user.to_owned();
                                            view!{
                                                button(on:click = move |_| {
                                                    let send = send.to_owned();
                                                    let user = user.to_owned();
                                                    console_dbg!(&user);
                                                    spawn_local(async move {
                                                        send.get_clone().unwrap().send(Message::Text(serde_json::to_string(&MessageText{ message_type: EventType::Restart, user }).unwrap())).await.unwrap();
                                                    });
                                                }){"Reset"}
                                            }
                                        },
                                        false => {
                                            let send = send.to_owned();
                                            let user = user.to_owned();
                                            view!{
                                                button(on:click = move |_|{
                                                    let send = send.to_owned();
                                                    let user = user.to_owned();
                                                    console_dbg!(&user);
                                                    spawn_local(async move {
                                                        send.get_clone().unwrap().send(Message::Text(serde_json::to_string(&MessageText{ message_type: EventType::Show, user }).unwrap())).await.unwrap();
                                                    });
                                                }){"Show cards"}
                                            }
                                        }
                                    })
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
                                        select(id="user_vote",bind:value=number){
                                            option(value = "", selected = true, disabled = true){"Your vote..."}
                                            option(value = "1"){"1"}
                                            option(value = "2"){"2"}
                                            option(value = "3"){"3"}
                                            option(value = "5"){"5"}
                                            option(value = "8"){"8"}
                                            option(value = "13"){"13"}
                                            option(value = "21"){"21"}
                                            option(value = "34"){"34"}
                                        }
                                        // input(r#type="number",bind:value=number, list="number_list"){}
                                        // datalist(id="number_list"){

                                        // }
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
                        aside(id="right"){
                            UserCards(users = split_users.get_clone().1.to_owned(), show = show, side = Side::Right)
                        }
                    }
                },
                true => view!{
                    article(id="code_message"){
                        p(){"Please give the connection code to the team so they can connect"}
                    }
                },
            })

        }
    }
}
