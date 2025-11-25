use crate::user::User;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct MessageText {
    pub message_type: EventType,
    pub user: User,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct MessageBack {
    pub room: String,
    pub show: bool,
    pub users: Vec<User>,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, Default)]
pub enum EventType {
    Start,
    #[default]
    Login,
    Select,
    Show,
    Restart,
}
