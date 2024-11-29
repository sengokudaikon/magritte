//! Time functions for SurrealDB queries
//!
//! These functions can be used when working with and manipulating datetime
//! values.

use std::fmt::{self, Display};

use super::Callable;

/// Time function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum TimeFunction {
    /// Rounds a datetime up to the next largest duration
    Ceil(String, String), // datetime, duration
    /// Extracts the day as a number from a datetime
    Day(Option<String>), // optional datetime, uses current if None
    /// Rounds a datetime down by a specific duration
    Floor(String, String), // datetime, duration
    /// Outputs a datetime according to a specific format
    Format(String, String), // datetime, format
    /// Groups a datetime by a particular time interval
    Group(String, String), // datetime, interval
    /// Extracts the hour as a number from a datetime
    Hour(Option<String>),
    /// Finds the most recent datetime in an array
    Max(String),
    /// Extracts the microseconds as a number from a datetime
    Micros(Option<String>),
    /// Extracts the milliseconds as a number from a datetime
    Millis(Option<String>),
    /// Finds the least recent datetime in an array
    Min(String),
    /// Extracts the minutes as a number from a datetime
    Minute(Option<String>),
    /// Extracts the month as a number from a datetime
    Month(Option<String>),
    /// Returns the current datetime
    Now,
    /// Extracts the nanoseconds as a number from a datetime
    Nanos(Option<String>),
    /// Returns the current datetime in RFC3339 format
    Round(String, String), // datetime, duration
    /// Extracts the seconds as a number from a datetime
    Second(Option<String>),
    /// Extracts the timezone as a string from a datetime
    Timezone(Option<String>),
    /// Extracts the unix timestamp in microseconds from a datetime
    UnixMicros(Option<String>),
    /// Extracts the unix timestamp in milliseconds from a datetime
    UnixMillis(Option<String>),
    /// Extracts the unix timestamp in nanoseconds from a datetime
    UnixNanos(Option<String>),
    /// Extracts the unix timestamp in seconds from a datetime
    UnixSeconds(Option<String>),
    /// Extracts the week as a number from a datetime
    Wday(Option<String>),
    /// Extracts the week as a number from a datetime
    Week(Option<String>),
    /// Extracts the year as a number from a datetime
    Year(Option<String>),
    /// Extracts the day of the year as a number from a datetime
    Yday(Option<String>),

    // From functions
    /// Converts microseconds to datetime
    FromMicros(String),
    /// Converts milliseconds to datetime
    FromMillis(String),
    /// Converts nanoseconds to datetime
    FromNanos(String),
    /// Converts seconds to datetime
    FromSecs(String),
    /// Converts seconds to datetime (alias for FromSecs)
    FromUnix(String),
}

impl Display for TimeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ceil(dt, dur) => write!(f, "time::ceil({}, {})", dt, dur),
            Self::Day(dt) => {
                match dt {
                    Some(d) => write!(f, "time::day({})", d),
                    None => write!(f, "time::day()"),
                }
            }
            Self::Floor(dt, dur) => write!(f, "time::floor({}, {})", dt, dur),
            Self::Format(dt, fmt) => write!(f, "time::format({}, {})", dt, fmt),
            Self::Group(dt, interval) => write!(f, "time::group({}, {})", dt, interval),
            Self::Hour(dt) => {
                match dt {
                    Some(d) => write!(f, "time::hour({})", d),
                    None => write!(f, "time::hour()"),
                }
            }
            Self::Max(arr) => write!(f, "time::max({})", arr),
            Self::Micros(dt) => {
                match dt {
                    Some(d) => write!(f, "time::micros({})", d),
                    None => write!(f, "time::micros()"),
                }
            }
            Self::Millis(dt) => {
                match dt {
                    Some(d) => write!(f, "time::millis({})", d),
                    None => write!(f, "time::millis()"),
                }
            }
            Self::Min(arr) => write!(f, "time::min({})", arr),
            Self::Minute(dt) => {
                match dt {
                    Some(d) => write!(f, "time::minute({})", d),
                    None => write!(f, "time::minute()"),
                }
            }
            Self::Month(dt) => {
                match dt {
                    Some(d) => write!(f, "time::month({})", d),
                    None => write!(f, "time::month()"),
                }
            }
            Self::Now => write!(f, "time::now()"),
            Self::Nanos(dt) => {
                match dt {
                    Some(d) => write!(f, "time::nanos({})", d),
                    None => write!(f, "time::nanos()"),
                }
            }
            Self::Round(dt, dur) => write!(f, "time::round({}, {})", dt, dur),
            Self::Second(dt) => {
                match dt {
                    Some(d) => write!(f, "time::second({})", d),
                    None => write!(f, "time::second()"),
                }
            }
            Self::Timezone(dt) => {
                match dt {
                    Some(d) => write!(f, "time::timezone({})", d),
                    None => write!(f, "time::timezone()"),
                }
            }
            Self::UnixMicros(dt) => {
                match dt {
                    Some(d) => write!(f, "time::unix::micros({})", d),
                    None => write!(f, "time::unix::micros()"),
                }
            }
            Self::UnixMillis(dt) => {
                match dt {
                    Some(d) => write!(f, "time::unix::millis({})", d),
                    None => write!(f, "time::unix::millis()"),
                }
            }
            Self::UnixNanos(dt) => {
                match dt {
                    Some(d) => write!(f, "time::unix::nanos({})", d),
                    None => write!(f, "time::unix::nanos()"),
                }
            }
            Self::UnixSeconds(dt) => {
                match dt {
                    Some(d) => write!(f, "time::unix({})", d),
                    None => write!(f, "time::unix()"),
                }
            }
            Self::Wday(dt) => {
                match dt {
                    Some(d) => write!(f, "time::wday({})", d),
                    None => write!(f, "time::wday()"),
                }
            }
            Self::Week(dt) => {
                match dt {
                    Some(d) => write!(f, "time::week({})", d),
                    None => write!(f, "time::week()"),
                }
            }
            Self::Year(dt) => {
                match dt {
                    Some(d) => write!(f, "time::year({})", d),
                    None => write!(f, "time::year()"),
                }
            }
            Self::Yday(dt) => {
                match dt {
                    Some(d) => write!(f, "time::yday({})", d),
                    None => write!(f, "time::yday()"),
                }
            }
            Self::FromMicros(micros) => write!(f, "time::from::micros({})", micros),
            Self::FromMillis(millis) => write!(f, "time::from::millis({})", millis),
            Self::FromNanos(nanos) => write!(f, "time::from::nanos({})", nanos),
            Self::FromSecs(secs) => write!(f, "time::from::secs({})", secs),
            Self::FromUnix(secs) => write!(f, "time::from::unix({})", secs),
        }
    }
}

impl Callable for TimeFunction {
    fn namespace() -> &'static str { "time" }

    fn category(&self) -> &'static str {
        match self {
            // Current time
            Self::Now => "current",

            // Time components extraction
            Self::Day(..)
            | Self::Hour(..)
            | Self::Minute(..)
            | Self::Month(..)
            | Self::Second(..)
            | Self::Week(..)
            | Self::Year(..)
            | Self::Yday(..)
            | Self::Wday(..) => "component",

            // Precision components
            Self::Micros(..) | Self::Millis(..) | Self::Nanos(..) => "precision",

            // Unix timestamps
            Self::UnixMicros(..) | Self::UnixMillis(..) | Self::UnixNanos(..) | Self::UnixSeconds(..) => "unix",

            // Time manipulation
            Self::Ceil(..) | Self::Floor(..) | Self::Round(..) => "rounding",

            // Time formatting
            Self::Format(..) | Self::Group(..) => "format",

            // Time comparison
            Self::Max(..) | Self::Min(..) => "comparison",

            // Time zone
            Self::Timezone(..) => "timezone",

            // Time conversion
            Self::FromMicros(..)
            | Self::FromMillis(..)
            | Self::FromNanos(..)
            | Self::FromSecs(..)
            | Self::FromUnix(..) => "conversion",
        }
    }

    fn can_filter(&self) -> bool {
        false // Time functions return datetime or numeric values, not boolean
    }
}
