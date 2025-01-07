//! Type functions for SurrealDB queries
//!
//! These functions can be used for generating and coercing data to specific
//! data types.

use std::fmt::{self, Display};

use super::Callable;

/// Type function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum TypeFunction {
    /// Converts a value into an array
    Array(String),
    /// Converts a value into a boolean
    Bool(String),
    /// Converts a value into bytes
    Bytes(String),
    /// Converts a value into a datetime
    Datetime(String),
    /// Converts a value into a decimal
    Decimal(String),
    /// Converts a value into a duration
    Duration(String),
    /// Projects a single field within a SELECT statement
    Field(String),
    /// Projects multiple fields within a SELECT statement
    Fields(Vec<String>),
    /// Converts a value into a floating point number
    Float(String),
    /// Converts a value into an integer
    Int(String),
    /// Converts a value into a number
    Number(String),
    /// Converts a value into a geometry point
    Point(String),
    /// Converts a value into a string
    String(String),
    /// Converts a value into a Table
    Table(String),
    /// Creates a record ID from a Table name and ID
    Thing(String, String), // Table, id

    // Is functions
    /// Checks if a value is an array
    IsArray(String),
    /// Checks if a value is a boolean
    IsBool(String),
    /// Checks if a value is a datetime
    IsDatetime(String),
    /// Checks if a value is a decimal
    IsDecimal(String),
    /// Checks if a value is a duration
    IsDuration(String),
    /// Checks if a value is a float
    IsFloat(String),
    /// Checks if a value is an integer
    IsInt(String),
    /// Checks if a value is null
    IsNull(String),
    /// Checks if a value is a number
    IsNumber(String),
    /// Checks if a value is an object
    IsObject(String),
    /// Checks if a value is a point
    IsPoint(String),
    /// Checks if a value is a record ID
    IsRecord(String, Option<String>), // value, optional Table
    /// Checks if a value is a string
    IsString(String),
    /// Checks if a value is a UUID
    IsUuid(String),
}

impl Display for TypeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Conversion functions
            Self::Array(val) => write!(f, "type::array({})", val),
            Self::Bool(val) => write!(f, "type::bool({})", val),
            Self::Bytes(val) => write!(f, "type::bytes({})", val),
            Self::Datetime(val) => write!(f, "type::datetime({})", val),
            Self::Decimal(val) => write!(f, "type::decimal({})", val),
            Self::Duration(val) => write!(f, "type::duration({})", val),
            Self::Field(val) => write!(f, "type::field({})", val),
            Self::Fields(fields) => write!(f, "type::fields([{}])", fields.join(", ")),
            Self::Float(val) => write!(f, "type::float({})", val),
            Self::Int(val) => write!(f, "type::int({})", val),
            Self::Number(val) => write!(f, "type::number({})", val),
            Self::Point(val) => write!(f, "type::point({})", val),
            Self::String(val) => write!(f, "type::string({})", val),
            Self::Table(val) => write!(f, "type::Table({})", val),
            Self::Thing(table, id) => write!(f, "type::thing({}, {})", table, id),

            // Is functions
            Self::IsArray(val) => write!(f, "type::is::array({})", val),
            Self::IsBool(val) => write!(f, "type::is::bool({})", val),
            Self::IsDatetime(val) => write!(f, "type::is::datetime({})", val),
            Self::IsDecimal(val) => write!(f, "type::is::decimal({})", val),
            Self::IsDuration(val) => write!(f, "type::is::duration({})", val),
            Self::IsFloat(val) => write!(f, "type::is::float({})", val),
            Self::IsInt(val) => write!(f, "type::is::int({})", val),
            Self::IsNull(val) => write!(f, "type::is::null({})", val),
            Self::IsNumber(val) => write!(f, "type::is::number({})", val),
            Self::IsObject(val) => write!(f, "type::is::object({})", val),
            Self::IsPoint(val) => write!(f, "type::is::point({})", val),
            Self::IsRecord(val, table) => match table {
                Some(t) => write!(f, "type::is::record({}, {})", val, t),
                None => write!(f, "type::is::record({})", val),
            },
            Self::IsString(val) => write!(f, "type::is::string({})", val),
            Self::IsUuid(val) => write!(f, "type::is::uuid({})", val),
        }
    }
}

impl Callable for TypeFunction {
    fn namespace() -> &'static str {
        "type"
    }

    fn category(&self) -> &'static str {
        match self {
            // Conversion functions
            Self::Array(..)
            | Self::Bool(..)
            | Self::Bytes(..)
            | Self::Datetime(..)
            | Self::Decimal(..)
            | Self::Duration(..)
            | Self::Float(..)
            | Self::Int(..)
            | Self::Number(..)
            | Self::Point(..)
            | Self::String(..)
            | Self::Table(..)
            | Self::Thing(..) => "conversion",

            // Field projection
            Self::Field(..) | Self::Fields(..) => "projection",

            // Type checking
            Self::IsArray(..)
            | Self::IsBool(..)
            | Self::IsDatetime(..)
            | Self::IsDecimal(..)
            | Self::IsDuration(..)
            | Self::IsFloat(..)
            | Self::IsInt(..)
            | Self::IsNull(..)
            | Self::IsNumber(..)
            | Self::IsObject(..)
            | Self::IsPoint(..)
            | Self::IsRecord(..)
            | Self::IsString(..)
            | Self::IsUuid(..) => "validation",
        }
    }

    fn can_filter(&self) -> bool {
        matches!(
            self,
            // All is functions can be used in WHERE
            Self::IsArray(..)
                | Self::IsBool(..)
                | Self::IsDatetime(..)
                | Self::IsDecimal(..)
                | Self::IsDuration(..)
                | Self::IsFloat(..)
                | Self::IsInt(..)
                | Self::IsNull(..)
                | Self::IsNumber(..)
                | Self::IsObject(..)
                | Self::IsPoint(..)
                | Self::IsRecord(..)
                | Self::IsString(..)
                | Self::IsUuid(..)
        )
    }
}
