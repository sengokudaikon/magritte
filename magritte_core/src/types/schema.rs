use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SchemaType {
    #[default]
    Schemafull,
    Schemaless,
}
impl FromStr for SchemaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SchemaType::from(s.to_string()))
    }
}
impl From<&str> for SchemaType {
    fn from(value: &str) -> Self {
        SchemaType::from(value.to_string())
    }
}
impl From<String> for SchemaType {
    fn from(s: String) -> Self {
        SchemaType::from(&s)
    }
}
impl From<&String> for SchemaType {
    fn from(value: &String) -> Self {
        match value.to_lowercase().as_str() {
            "schemafull" => SchemaType::Schemafull,
            "schemaless" => SchemaType::Schemaless,
            _ => panic!("Invalid schema type"),
        }
    }
}
impl Display for SchemaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SchemaType::Schemafull => write!(f, "SCHEMAFULL"),
            SchemaType::Schemaless => write!(f, "SCHEMALESS"),
        }
    }
}
