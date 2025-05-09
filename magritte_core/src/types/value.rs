use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum SqlValue {
    Null,
    Raw(String),
    Param(String),
    Value(Value),
}
impl Display for SqlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlValue::Raw(s) => write!(f, "{}", s),
            SqlValue::Param(p) => write!(f, "${}", p),
            SqlValue::Value(v) => write!(f, "{}", v),
            SqlValue::Null => write!(f, ""),
        }
    }
}
