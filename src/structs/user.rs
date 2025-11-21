use serde::{Deserialize, Serialize};
use crate::string;

#[derive(Clone, Deserialize, Serialize, Debug, Default, Eq, Hash)]
pub struct User {
    role: Role,
    name: String,
    value: Option<u8>
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl User {
    pub fn new(role: Role, name: &str, value: Option<u8>) -> User {
        User {
            role,
            name: string!(name),
            value,
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
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, Default, Eq, PartialEq, Hash)]
pub enum Role {
    Master,
    #[default]
    Voter,
}

impl Into<Role> for String {
    fn into(self) -> Role {
        match self.as_str() {
            "Master" => Role::Master,
            "Voter" => Role::Voter,
            _ => unreachable!()
        }
    }
}