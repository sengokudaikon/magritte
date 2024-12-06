use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub enum Operator {
    Eq,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    NotEq,
    Inside,      // For array/set containment
    Outside,     // For array/set exclusion
    Contains,    // For array/set membership
    ContainsAll, // For checking if all elements exist
    ContainsAny, // For checking if any elements exist
    Raw,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Condition {
    And(Vec<String>),
    Or(Vec<String>),
    Raw(String),
    Param(String, String), // field name, parameter name
}

impl From<Operator> for String {
    fn from(value: Operator) -> Self {
        match value {
            Operator::Eq => "=".into(),
            Operator::Gt => ">".into(),
            Operator::Lt => "<".into(),
            Operator::Gte => ">=".into(),
            Operator::Lte => "<=".into(),
            Operator::Like => "LIKE".into(),
            Operator::NotEq => "!=".into(),
            Operator::Inside => "INSIDE".into(),
            Operator::Outside => "OUTSIDE".into(),
            Operator::Contains => "CONTAINS".into(),
            Operator::ContainsAll => "CONTAINSALL".into(),
            Operator::ContainsAny => "CONTAINSANY".into(),
            Operator::Raw => "".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_to_string() {
        assert_eq!(String::from(Operator::Eq), "=");
        assert_eq!(String::from(Operator::NotEq), "!=");
        assert_eq!(String::from(Operator::Contains), "CONTAINS");
    }
}
