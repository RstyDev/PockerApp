
use futures::channel::mpsc::UnboundedSender;
use gloo_net::websocket::Message;
use sycamore::prelude::*;
use macros::string;
use structs::{Role, User};

#[component(inline_props)]
pub fn Table(
    user: User,
    users: Signal<Vec<User>>,
    ws_sender: Signal<Option<UnboundedSender<Message>>>,
) -> View {
    let master = users.get_clone().into_iter().find(|user|user.role()==Role::Master);
    // let view = users.get_clone().into_iter().map(|user|{view!{article(){(user.value().unwrap_or_default().to_string())}}}).collect::<Vec<View>>();
    // let cards = users.get_clone().into_iter().map(|user| view!{article(class="card"){(user.value().unwrap_or_default().to_string())}}).collect<Vec<View>>();
    view! {
        (match user.role(){
            Role::Master => format!("Connection code: {}",master.as_ref().map(|m|m.room().to_string()).unwrap_or_default()),
            Role::Voter => string!(""),
        })
        Indexed(
            list=users,
            view=|user| view!{
                article(){
                    (match user.role(){
                        Role::Master => user.name().to_string(),
                        Role::Voter => format!("{}: {}",user.name(),user.value().map(|v|v.to_string()).unwrap_or_default())
                    })
                }
            }
        )
    }
}
