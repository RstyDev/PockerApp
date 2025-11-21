use crate::structs::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    NotLogged,
    Logged(User),
}
