use serde::{Deserialize, Serialize};
use crate::structs::User;

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct MessageText {
    pub message_type: EventType,
    pub room: String,
    pub user: User,
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
