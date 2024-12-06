use std::fmt::Display;
use std::time::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::sql::Algorithm;

#[derive(Debug, Serialize, Deserialize, Hash, Clone, Eq, PartialEq, PartialOrd)]
pub enum AccessType {
    Record {
        signup: Option<String>,
        signin: Option<String>,
        with: Option<Box<AccessType>>,
    },
    Jwt {
        algorithm: Option<Algorithm>,
        key: Option<String>,
        url: Option<String>,
        with_issuer: Option<String>,
    }
}

impl Default for AccessType {
    fn default() -> Self {
        Self::Record {
            signup: None,
            signin: None,
            with: None
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Hash, Clone, Eq, PartialEq, PartialOrd, Default)]
pub enum AccessTargets {
    #[default]
    Root,
    Namespace,
    Database,
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct DefineAccessStatement {
    pub name: String,
    pub if_not_exists: bool,
    pub overwrite: bool,
    pub on: AccessTargets,
    pub ac_type: AccessType,
    pub authenticate: Option<String>,
    pub for_session: Option<Duration>,
    pub for_token: Option<Duration>,
    pub comment: Option<String>,
}

impl DefineAccessStatement {
    pub fn new() -> Self {
        Self::default()
    }
}
