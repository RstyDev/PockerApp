use futures::channel::mpsc::UnboundedSender;
use gloo_net::websocket::Message;
use sycamore::prelude::*;
use crate::structs::{User, Role};

#[component(inline_props)]
pub fn Table(user: User, users: Signal<Vec<User>>, ws_sender: Signal<Option<UnboundedSender<Message>>>) -> View {
    // let view = users.get_clone().into_iter().map(|user|{view!{article(){(user.value().unwrap_or_default().to_string())}}}).collect::<Vec<View>>();
    // let cards = users.get_clone().into_iter().map(|user| view!{article(class="card"){(user.value().unwrap_or_default().to_string())}}).collect<Vec<View>>();
    view!{
        Indexed(
            list=users,
            view=|user| view!{
                article(){
                    (match user.role(){
                        Role::Master => String::new(),
                        Role::Voter => user.value().unwrap_or_default().to_string()
                    })
                }
            }
        )
    }
}