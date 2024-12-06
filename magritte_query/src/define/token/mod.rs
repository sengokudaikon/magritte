use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq, PartialOrd)]
pub struct DefineTokenStatement {
    pub name: String,
    pub value: String,
}