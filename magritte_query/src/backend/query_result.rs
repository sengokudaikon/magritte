//! Query result types for SurrealDB queries

use magritte_core::{RangeTarget, RecordType, SurrealId};
use serde_json::Value;
use std::fmt::{self, Debug, Display};

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
pub enum FromTarget<T>
where
    T: RecordType,
{
    /// A Table name
    Table(String),
    Record(SurrealId<T>), // (Table, id)
    /// A list of record IDs
    RecordList(Vec<SurrealId<T>>),
    /// A range of record IDs
    Range(String, RangeTarget),
    /// A subquery
    Subquery(QueryResult),
    /// A dynamic Table reference
    Dynamic(String), // e.g., "type::Table($Table)"
}

impl<T> Display for FromTarget<T>
where
    T: RecordType,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromTarget::Table(name) => write!(f, "{}", name),
            FromTarget::Record(record_id) => write!(f, "{}", record_id),
            FromTarget::RecordList(records) => {
                write!(
                    f,
                    "[{}]",
                    records
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            FromTarget::Range(name, range) => write!(f, "{}:{}", name, range),
            FromTarget::Subquery(qr) => write!(f, "({})", qr),
            FromTarget::Dynamic(expr) => write!(f, "{}", expr),
        }
    }
}
