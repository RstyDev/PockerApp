use macros::string;
use structs::{Role, User};
use sycamore::prelude::*;
#[derive(Clone, Copy, Debug, Default)]
pub enum Side {
    #[default]
    Left,
    Right,
}
#[component(inline_props)]
pub fn UserCards(
    mut users: Vec<User>,
    this_user: ReadSignal<User>,
    show: Signal<bool>,
    side: Side,
) -> View {
    let show = create_selector(move || show.get());

    users.sort_by(|u, o| u.name().cmp(o.name()));
    let views = users
        .into_iter()
        .map(|user| {
            let name = user.name().to_string();
            let name2 = name.clone();
            let role = user.role();
            let value = user.value();
            let is_current = this_user.with_untracked(|u|u.name().eq(user.name()));
            view! {
                article(class=role.to_string()){
                    (match side{
                        Side::Left => view!{
                            span(){
                                (name)
                            }
                        },
                        Side::Right => view!{}
                    })
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
                                        (match is_current {
                                            true => value.map(|v|v.to_string()).unwrap_or_default(),
                                            false => if show.get(){value.map(|v|v.to_string()).unwrap_or_default()} else {string!("")},
                                        })

                                    }
                                }
                            }
                        },
                    })
                    (match side {
                        Side::Right => view!{
                            span(){
                                (name2)
                            }
                        },
                        Side::Left => view!{}
                    })
                }
            }
        })
        .collect::<Vec<View>>();
    view! {
        (views)
    }
}
