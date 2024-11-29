use std::fmt;
use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
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
