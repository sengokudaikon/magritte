//! HTTP functions for SurrealDB queries
//!
//! These functions can be used to make HTTP requests to external services.

use std::fmt::{self, Display};

use super::Callable;

/// HTTP function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum HttpFunction {
    /// Performs a DELETE request
    Delete(String, Option<String>, Option<String>),
    /// Performs a GET request
    Get(String, Option<String>),
    /// Performs a HEAD request
    Head(String, Option<String>),
    /// Performs a PATCH request
    Patch(String, Option<String>, Option<String>),
    /// Performs a POST request
    Post(String, Option<String>, Option<String>),
    /// Performs a PUT request
    Put(String, Option<String>, Option<String>),
}

impl Display for HttpFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Delete(url, headers, body) => {
                if let Some(headers) = headers {
                    if let Some(body) = body {
                        write!(f, "http::delete({}, {}, {})", url, headers, body)
                    } else {
                        write!(f, "http::delete({}, {})", url, headers)
                    }
                } else {
                    write!(f, "http::delete({})", url)
                }
            }
            Self::Get(url, headers) => {
                if let Some(headers) = headers {
                    write!(f, "http::get({}, {})", url, headers)
                } else {
                    write!(f, "http::get({})", url)
                }
            }
            Self::Head(url, headers) => {
                if let Some(headers) = headers {
                    write!(f, "http::head({}, {})", url, headers)
                } else {
                    write!(f, "http::head({})", url)
                }
            }
            Self::Patch(url, headers, body) => {
                if let Some(headers) = headers {
                    if let Some(body) = body {
                        write!(f, "http::patch({}, {}, {})", url, headers, body)
                    } else {
                        write!(f, "http::patch({}, {})", url, headers)
                    }
                } else {
                    write!(f, "http::patch({})", url)
                }
            }
            Self::Post(url, headers, body) => {
                if let Some(headers) = headers {
                    if let Some(body) = body {
                        write!(f, "http::post({}, {}, {})", url, headers, body)
                    } else {
                        write!(f, "http::post({}, {})", url, headers)
                    }
                } else {
                    write!(f, "http::post({})", url)
                }
            }
            Self::Put(url, headers, body) => {
                if let Some(headers) = headers {
                    if let Some(body) = body {
                        write!(f, "http::put({}, {}, {})", url, headers, body)
                    } else {
                        write!(f, "http::put({}, {})", url, headers)
                    }
                } else {
                    write!(f, "http::put({})", url)
                }
            }
        }
    }
}

impl Callable for HttpFunction {
    fn namespace() -> &'static str {
        "http"
    }

    fn category(&self) -> &'static str {
        match self {
            // GET and HEAD are read operations
            Self::Get(..) | Self::Head(..) => "read",

            // POST, PUT, PATCH, DELETE are write operations
            Self::Post(..) | Self::Put(..) | Self::Patch(..) | Self::Delete(..) => "write",
        }
    }

    fn can_filter(&self) -> bool {
        false // HTTP functions return response data, not boolean conditions
    }
}
