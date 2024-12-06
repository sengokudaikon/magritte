use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq, PartialOrd)]
pub struct DefineConfigStatement;
