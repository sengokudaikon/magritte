//! Duration functions for SurrealDB queries

use std::fmt::{self, Display};

use super::Callable;

/// Duration function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum DurationFunction {
    /// Counts how many days fit in a duration
    Days(String),
    /// Counts how many hours fit in a duration
    Hours(String),
    /// Counts how many microseconds fit in a duration
    Micros(String),
    /// Counts how many milliseconds fit in a duration
    Millis(String),
    /// Counts how many minutes fit in a duration
    Mins(String),
    /// Counts how many nanoseconds fit in a duration
    Nanos(String),
    /// Counts how many seconds fit in a duration
    Secs(String),
    /// Counts how many weeks fit in a duration
    Weeks(String),
    /// Counts how many years fit in a duration
    Years(String),

    // From functions
    /// Converts days to duration
    FromDays(String),
    /// Converts hours to duration
    FromHours(String),
    /// Converts microseconds to duration
    FromMicros(String),
    /// Converts milliseconds to duration
    FromMillis(String),
    /// Converts minutes to duration
    FromMins(String),
    /// Converts nanoseconds to duration
    FromNanos(String),
    /// Converts seconds to duration
    FromSecs(String),
    /// Converts weeks to duration
    FromWeeks(String),
}

impl Display for DurationFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Basic duration functions
            Self::Days(val) => write!(f, "duration::days({})", val),
            Self::Hours(val) => write!(f, "duration::hours({})", val),
            Self::Micros(val) => write!(f, "duration::micros({})", val),
            Self::Millis(val) => write!(f, "duration::millis({})", val),
            Self::Mins(val) => write!(f, "duration::mins({})", val),
            Self::Nanos(val) => write!(f, "duration::nanos({})", val),
            Self::Secs(val) => write!(f, "duration::secs({})", val),
            Self::Weeks(val) => write!(f, "duration::weeks({})", val),
            Self::Years(val) => write!(f, "duration::years({})", val),

            // From functions
            Self::FromDays(val) => write!(f, "duration::from::days({})", val),
            Self::FromHours(val) => write!(f, "duration::from::hours({})", val),
            Self::FromMicros(val) => write!(f, "duration::from::micros({})", val),
            Self::FromMillis(val) => write!(f, "duration::from::millis({})", val),
            Self::FromMins(val) => write!(f, "duration::from::mins({})", val),
            Self::FromNanos(val) => write!(f, "duration::from::nanos({})", val),
            Self::FromSecs(val) => write!(f, "duration::from::secs({})", val),
            Self::FromWeeks(val) => write!(f, "duration::from::weeks({})", val),
        }
    }
}

impl Callable for DurationFunction {
    fn namespace() -> &'static str { "duration" }

    fn category(&self) -> &'static str {
        match self {
            // Basic duration functions
            Self::Days(..)
            | Self::Hours(..)
            | Self::Micros(..)
            | Self::Millis(..)
            | Self::Mins(..)
            | Self::Nanos(..)
            | Self::Secs(..)
            | Self::Weeks(..)
            | Self::Years(..) => "conversion",

            // From functions
            Self::FromDays(..)
            | Self::FromHours(..)
            | Self::FromMicros(..)
            | Self::FromMillis(..)
            | Self::FromMins(..)
            | Self::FromNanos(..)
            | Self::FromSecs(..)
            | Self::FromWeeks(..) => "from",
        }
    }

    fn can_filter(&self) -> bool {
        false // Duration functions return numeric values, not boolean
    }
}
