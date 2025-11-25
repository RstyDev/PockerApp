use macros::string;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default, Eq, Hash)]
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
    pub fn new(role: Role, name: &str, value: Option<u8>, room: String) -> User {
        User {
            role,
            name: string!(name),
            value,
            room,
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

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Master => string!("Master"),
            Role::Voter => string!("Voter"),
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

impl Into<Role> for String {
    fn into(self) -> Role {
        match self.as_str() {
            "Master" => Role::Master,
            "Voter" => Role::Voter,
            _ => unreachable!(),
        }
    }
}
