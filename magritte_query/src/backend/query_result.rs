//! Query result types for SurrealDB queries

use std::fmt::{self, Display};

use crate::types::RangeTarget;
use serde_json::Value;

/// Represents a query result or subquery specification
#[derive(Clone, Debug, PartialEq)]
pub enum QueryResult {
    /// A raw SQL query Str
    Raw(String),
    /// A field projection or computed value
    Field { expr: String, alias: Option<String> },
    /// A subquery result
    Subquery {
        query: String,
        alias: Option<String>,
    },
    /// An array or object value
    Value(Value),
}

impl Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryResult::Raw(sql) => write!(f, "{}", sql),
            QueryResult::Field { expr, alias } => {
                if let Some(alias) = alias {
                    write!(f, "{} AS {}", expr, alias)
                } else {
                    write!(f, "{}", expr)
                }
            }
            QueryResult::Subquery { query, alias } => {
                if let Some(alias) = alias {
                    write!(f, "({}) AS {}", query, alias)
                } else {
                    write!(f, "({})", query)
                }
            }
            QueryResult::Value(val) => write!(f, "{}", val),
        }
    }
}

/// Represents a target in a FROM clause
#[derive(Clone, Debug, PartialEq)]
pub enum FromTarget {
    /// A Table name
    Table(String),
    Record(String, String), // (Table, id)
    /// A list of record IDs
    RecordList(Vec<String>),
    /// A range of record IDs
    Range(String, RangeTarget),
    /// A subquery
    Subquery(QueryResult),
    /// A dynamic Table reference
    Dynamic(String), // e.g., "type::Table($Table)"
}

impl Display for FromTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromTarget::Table(name) => write!(f, "{}", name),
            FromTarget::Record(table, id) => write!(f, "{}:{}", table, id),
            FromTarget::RecordList(records) => write!(f, "[{}]", records.join(", ")),
            FromTarget::Range(name, range) => write!(f, "{}:{}", name, range),
            FromTarget::Subquery(qr) => write!(f, "({})", qr),
            FromTarget::Dynamic(expr) => write!(f, "{}", expr),
        }
    }
}
