use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::types::schema::SchemaType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    None,
    Full,
    Select(String),
    Create(String),
    Update(String),
    Delete(String),
}
impl From<String> for Permission {
    fn from(value: String) -> Self {
        Permission::from(&value)
    }
}
impl From<&String> for Permission {
    fn from(value: &String) -> Self {
        let value = value.to_lowercase();
        if value.starts_with("for select where ") {
            let condition = value.trim_start_matches("for select where ");
            Permission::Select(condition.to_string())
        } else if value.starts_with("for create where ") {
            let condition = value.trim_start_matches("for create where ");
            Permission::Create(condition.to_string())
        } else if value.starts_with("for update where ") {
            let condition = value.trim_start_matches("for update where ");
            Permission::Update(condition.to_string())
        } else if value.starts_with("for delete where ") {
            let condition = value.trim_start_matches("for delete where ");
            Permission::Delete(condition.to_string())
        } else if value == "none" {
            Permission::None
        } else if value == "full" {
            Permission::Full
        } else {
            panic!("Invalid permission type")
        }
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Permission::None => write!(f, " NONE"),
            Permission::Full => write!(f, " FULL"),
            Permission::Select(v) => write!(f, " FOR select WHERE {}", v),
            Permission::Create(v) => write!(f, " FOR create WHERE {}", v),
            Permission::Update(v) => write!(f, " FOR update WHERE {}", v),
            Permission::Delete(v) => write!(f, " FOR delete WHERE {}", v),
        }
    }
}
