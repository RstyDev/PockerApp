use std::fmt::Display;

use macros::string;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default, Eq)]
pub struct User {
    role: Role,
    name: String,
    room: String,
    value: Option<u8>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl User {
    pub fn new(role: Role, name: &str, value: Option<u8>, room: &str) -> User {
        User {
            role,
            name: string!(name),
            value,
            room: string!(room),
        }
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> Option<u8> {
        self.value
    }

    pub fn set_role(&mut self, role: Role) {
        self.role = role;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_value(&mut self, value: Option<u8>) {
        self.value = value;
    }

    pub fn room(&self) -> &str {
        &self.room
    }

    pub fn set_room(&mut self, room: String) {
        self.room = room;
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, Default, Eq, PartialEq, Hash)]
pub enum Role {
    Master,
    #[default]
    Voter,
}

impl TryFrom<String> for Role {
    type Error = &'static str;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        match val.as_str() {
            "Master" => Ok(Role::Master),
            "Voter" => Ok(Role::Voter),
            &_ => Err("Invalid input for Role"),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "Master"),
            Role::Voter => write!(f, "Voter"),
        }
    }
}

// impl Into<String> for Role {
//     fn into(self) -> String {
//         match self {
//             Role::Master => string!("Master"),
//             Role::Voter => string!("Voter"),
//         }
//     }
// }

// impl Into<Role> for String {
//     fn into(self) -> Role {
//         match self.as_str() {
//             "Master" => Role::Master,
//             "Voter" => Role::Voter,
//             _ => unreachable!(),
//         }
//     }
// }
