use structs::{Role, User};
use sycamore::prelude::*;

#[component(inline_props)]
pub fn UserCards(mut users: Vec<User>, show: Signal<bool>) -> View {
    let show = create_selector(move || show.get());

    users.sort_by(|u, o| u.name().cmp(o.name()));
    let views = users
        .into_iter()
        .map(|user| {
            let name = user.name().to_string();
            let role = user.role();
            let value = user.value().clone();
            view! {
                article(class=role.to_string()){
                    span(){
                        (name)
                    }
                    (match role {
                        Role::Master => view!{},
                        Role::Voter => {
                            view!{
                                div(class = format!("card {}",match show.get(){
                                    true => "show",
                                    false => match value {
                                        Some(_) => "voted",
                                        None => "voting",
                                    }
                                })){
                                    p(){
                                        (show.get().then(||value.map(|v|v.to_string()).unwrap_or_default()).unwrap_or_default())
                                    }
                                }
                            }
                        },
                    })
                }
            }
        })
        .collect::<Vec<View>>();
    view! {
        (views)
    }
}
