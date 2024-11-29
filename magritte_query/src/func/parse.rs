//! Parse functions for SurrealDB queries
//!
//! These functions can be used when parsing email addresses and URL web
//! addresses.

use std::fmt::{self, Display};

use super::Callable;

/// Parse function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum ParseFunction {
    // Email functions
    /// Parses and returns an email host from an email address
    EmailHost(String),
    /// Parses and returns an email username from an email address
    EmailUser(String),

    // URL functions
    /// Parses and returns the domain from a URL
    UrlDomain(String),
    /// Parses and returns the fragment from a URL
    UrlFragment(String),
    /// Parses and returns the hostname from a URL
    UrlHost(String),
    /// Parses and returns the path from a URL
    UrlPath(String),
    /// Parses and returns the port number from a URL
    UrlPort(String),
    /// Parses and returns the scheme from a URL
    UrlScheme(String),
    /// Parses and returns the query string from a URL
    UrlQuery(String),
}

impl Display for ParseFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Email functions
            Self::EmailHost(email) => write!(f, "parse::email::host({})", email),
            Self::EmailUser(email) => write!(f, "parse::email::user({})", email),

            // URL functions
            Self::UrlDomain(url) => write!(f, "parse::url::domain({})", url),
            Self::UrlFragment(url) => write!(f, "parse::url::fragment({})", url),
            Self::UrlHost(url) => write!(f, "parse::url::host({})", url),
            Self::UrlPath(url) => write!(f, "parse::url::path({})", url),
            Self::UrlPort(url) => write!(f, "parse::url::port({})", url),
            Self::UrlScheme(url) => write!(f, "parse::url::scheme({})", url),
            Self::UrlQuery(url) => write!(f, "parse::url::query({})", url),
        }
    }
}

impl Callable for ParseFunction {
    fn namespace() -> &'static str { "parse" }

    fn category(&self) -> &'static str {
        match self {
            Self::EmailHost(..) | Self::EmailUser(..) => "email",
            Self::UrlDomain(..)
            | Self::UrlFragment(..)
            | Self::UrlHost(..)
            | Self::UrlPath(..)
            | Self::UrlPort(..)
            | Self::UrlScheme(..)
            | Self::UrlQuery(..) => "url",
        }
    }

    fn can_filter(&self) -> bool {
        false // Parse functions return strings or numbers, not boolean
    }
}
