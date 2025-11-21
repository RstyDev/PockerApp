use serde::{Deserialize, Serialize};
use crate::structs::User;

#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    NotLogged,
    Logged(User)
}