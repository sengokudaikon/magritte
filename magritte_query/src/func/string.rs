//! String functions for SurrealDB queries
//!
//! These functions can be used when working with and manipulating text and
//! string values.

use std::fmt::{self, Display};

use super::Callable;

/// String function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum StringFunction {
    // Basic string functions
    /// Concatenates strings together
    Concat(Vec<String>),
    /// Checks whether a string contains another string
    Contains(String, String),
    /// Checks whether a string ends with another string
    EndsWith(String, String),
    /// Joins strings together with a delimiter
    Join(String, String),
    /// Returns the length of a string
    Len(String),
    /// Converts a string to lowercase
    Lowercase(String),
    /// Performs a regex match on a string
    Matches(String, String),
    /// Repeats a string a number of times
    Repeat(String, usize),
    /// Replaces an occurrence of a string with another string
    Replace(String, String, String),
    /// Reverses a string
    Reverse(String),
    /// Extracts and returns a section of a string
    Slice(String, usize, usize),
    /// Converts a string into human and URL-friendly string
    Slug(String),
    /// Divides a string into an ordered list of substrings
    Split(String, String),
    /// Checks whether a string starts with another string
    StartsWith(String, String),
    /// Removes whitespace from the start and end of a string
    Trim(String),
    /// Converts a string to uppercase
    Uppercase(String),
    /// Splits a string into an array of separate words
    Words(String),

    // Distance functions
    /// Returns the Damerau–Levenshtein distance between two strings
    DamerauLevenshtein(String, String),
    /// Returns the normalized Damerau–Levenshtein distance between two strings
    NormalizedDamerauLevenshtein(String, String),
    /// Returns the Hamming distance between two strings
    Hamming(String, String),
    /// Returns the Levenshtein distance between two strings
    Levenshtein(String, String),
    /// Returns the normalized Levenshtein distance between two strings
    NormalizedLevenshtein(String, String),
    /// Returns the OSA distance between two strings
    OsaDistance(String, String),

    // HTML functions
    /// Encodes special characters into HTML entities
    HtmlEncode(String),
    /// Sanitizes HTML code
    HtmlSanitize(String),

    // Is functions
    /// Checks whether a value has only alphanumeric characters
    IsAlphanum(String),
    /// Checks whether a value has only alpha characters
    IsAlpha(String),
    /// Checks whether a value has only ascii characters
    IsAscii(String),
    /// Checks whether a string representation of a date and time matches a
    /// specified format
    IsDatetime(String, String),
    /// Checks whether a value is a domain
    IsDomain(String),
    /// Checks whether a value is an email
    IsEmail(String),
    /// Checks whether a value is hexadecimal
    IsHexadecimal(String),
    /// Checks whether a value is an IP address
    IsIp(String),
    /// Checks whether a value is an IP v4 address
    IsIpv4(String),
    /// Checks whether a value is an IP v6 address
    IsIpv6(String),
    /// Checks whether a value is a latitude value
    IsLatitude(String),
    /// Checks whether a value is a longitude value
    IsLongitude(String),
    /// Checks whether a value has only numeric characters
    IsNumeric(String),
    /// Checks whether a string is a Record ID, optionally of a certain Table
    IsRecord(String, Option<String>),
    /// Checks whether a value matches a semver version
    IsSemver(String),
    /// Checks whether a string is a ULID
    IsUlid(String),
    /// Checks whether a value is a valid URL
    IsUrl(String),
    /// Checks whether a string is a UUID
    IsUuid(String),

    // Semver functions
    /// Performs a comparison between two semver strings
    SemverCompare(String, String),
    /// Extract the major version from a semver string
    SemverMajor(String),
    /// Extract the minor version from a semver string
    SemverMinor(String),
    /// Extract the patch version from a semver string
    SemverPatch(String),
    /// Increment the major version of a semver string
    SemverIncMajor(String),
    /// Increment the minor version of a semver string
    SemverIncMinor(String),
    /// Increment the patch version of a semver string
    SemverIncPatch(String),
    /// Set the major version of a semver string
    SemverSetMajor(String, String),
    /// Set the minor version of a semver string
    SemverSetMinor(String, String),
    /// Set the patch version of a semver string
    SemverSetPatch(String, String),

    // Similarity functions
    /// Return the similarity score of fuzzy matching strings
    SimilarityFuzzy(String, String),
    /// Returns the Jaro similarity between two strings
    SimilarityJaro(String, String),
    /// Return the Jaro-Winkler similarity between two strings
    SimilarityJaroWinkler(String, String),
}

impl Display for StringFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Basic string functions
            Self::Concat(strings) => write!(f, "string::concat([{}])", strings.join(", ")),
            Self::Contains(str, substr) => write!(f, "string::contains({}, {})", str, substr),
            Self::EndsWith(str, substr) => write!(f, "string::ends_with({}, {})", str, substr),
            Self::Join(arr, sep) => write!(f, "string::join({}, {})", arr, sep),
            Self::Len(str) => write!(f, "string::len({})", str),
            Self::Lowercase(str) => write!(f, "string::lowercase({})", str),
            Self::Matches(str, pattern) => write!(f, "string::matches({}, {})", str, pattern),
            Self::Repeat(str, n) => write!(f, "string::repeat({}, {})", str, n),
            Self::Replace(str, old, new) => write!(f, "string::replace({}, {}, {})", str, old, new),
            Self::Reverse(str) => write!(f, "string::reverse({})", str),
            Self::Slice(str, start, end) => write!(f, "string::slice({}, {}, {})", str, start, end),
            Self::Slug(str) => write!(f, "string::slug({})", str),
            Self::Split(str, sep) => write!(f, "string::split({}, {})", str, sep),
            Self::StartsWith(str, substr) => write!(f, "string::starts_with({}, {})", str, substr),
            Self::Trim(str) => write!(f, "string::trim({})", str),
            Self::Uppercase(str) => write!(f, "string::uppercase({})", str),
            Self::Words(str) => write!(f, "string::words({})", str),

            // Distance functions
            Self::DamerauLevenshtein(s1, s2) => write!(f, "string::distance::damerau_levenshtein({}, {})", s1, s2),
            Self::NormalizedDamerauLevenshtein(s1, s2) => {
                write!(f, "string::distance::normalized_damerau_levenshtein({}, {})", s1, s2)
            }
            Self::Hamming(s1, s2) => write!(f, "string::distance::hamming({}, {})", s1, s2),
            Self::Levenshtein(s1, s2) => write!(f, "string::distance::levenshtein({}, {})", s1, s2),
            Self::NormalizedLevenshtein(s1, s2) => {
                write!(f, "string::distance::normalized_levenshtein({}, {})", s1, s2)
            }
            Self::OsaDistance(s1, s2) => write!(f, "string::distance::osa_distance({}, {})", s1, s2),

            // HTML functions
            Self::HtmlEncode(str) => write!(f, "string::html::encode({})", str),
            Self::HtmlSanitize(str) => write!(f, "string::html::sanitize({})", str),

            // Is functions
            Self::IsAlphanum(str) => write!(f, "string::is::alphanum({})", str),
            Self::IsAlpha(str) => write!(f, "string::is::alpha({})", str),
            Self::IsAscii(str) => write!(f, "string::is::ascii({})", str),
            Self::IsDatetime(str, fmt) => write!(f, "string::is::datetime({}, {})", str, fmt),
            Self::IsDomain(str) => write!(f, "string::is::domain({})", str),
            Self::IsEmail(str) => write!(f, "string::is::email({})", str),
            Self::IsHexadecimal(str) => write!(f, "string::is::hexadecimal({})", str),
            Self::IsIp(str) => write!(f, "string::is::ip({})", str),
            Self::IsIpv4(str) => write!(f, "string::is::ipv4({})", str),
            Self::IsIpv6(str) => write!(f, "string::is::ipv6({})", str),
            Self::IsLatitude(str) => write!(f, "string::is::latitude({})", str),
            Self::IsLongitude(str) => write!(f, "string::is::longitude({})", str),
            Self::IsNumeric(str) => write!(f, "string::is::numeric({})", str),
            Self::IsRecord(str, table) => {
                match table {
                    Some(t) => write!(f, "string::is::record({}, {})", str, t),
                    None => write!(f, "string::is::record({})", str),
                }
            }
            Self::IsSemver(str) => write!(f, "string::is::semver({})", str),
            Self::IsUlid(str) => write!(f, "string::is::ulid({})", str),
            Self::IsUrl(str) => write!(f, "string::is::url({})", str),
            Self::IsUuid(str) => write!(f, "string::is::uuid({})", str),

            // Semver functions
            Self::SemverCompare(v1, v2) => write!(f, "string::semver::compare({}, {})", v1, v2),
            Self::SemverMajor(ver) => write!(f, "string::semver::major({})", ver),
            Self::SemverMinor(ver) => write!(f, "string::semver::minor({})", ver),
            Self::SemverPatch(ver) => write!(f, "string::semver::patch({})", ver),
            Self::SemverIncMajor(ver) => write!(f, "string::semver::inc::major({})", ver),
            Self::SemverIncMinor(ver) => write!(f, "string::semver::inc::minor({})", ver),
            Self::SemverIncPatch(ver) => write!(f, "string::semver::inc::patch({})", ver),
            Self::SemverSetMajor(ver, val) => write!(f, "string::semver::set::major({}, {})", ver, val),
            Self::SemverSetMinor(ver, val) => write!(f, "string::semver::set::minor({}, {})", ver, val),
            Self::SemverSetPatch(ver, val) => write!(f, "string::semver::set::patch({}, {})", ver, val),

            // Similarity functions
            Self::SimilarityFuzzy(s1, s2) => write!(f, "string::similarity::fuzzy({}, {})", s1, s2),
            Self::SimilarityJaro(s1, s2) => write!(f, "string::similarity::jaro({}, {})", s1, s2),
            Self::SimilarityJaroWinkler(s1, s2) => write!(f, "string::similarity::jaro_winkler({}, {})", s1, s2),
        }
    }
}

impl Callable for StringFunction {
    fn namespace() -> &'static str { "string" }

    fn category(&self) -> &'static str {
        match self {
            // Basic string operations
            Self::Concat(..)
            | Self::Join(..)
            | Self::Repeat(..)
            | Self::Replace(..)
            | Self::Reverse(..)
            | Self::Slice(..)
            | Self::Split(..)
            | Self::Trim(..)
            | Self::Words(..) => "manipulation",

            // Case conversion
            Self::Lowercase(..) | Self::Uppercase(..) => "case",

            // String checks
            Self::Contains(..) | Self::EndsWith(..) | Self::StartsWith(..) | Self::Matches(..) => "check",

            // String analysis
            Self::Len(..) => "analysis",

            // URL/Slug handling
            Self::Slug(..) => "url",

            // Distance calculations
            Self::DamerauLevenshtein(..)
            | Self::NormalizedDamerauLevenshtein(..)
            | Self::Hamming(..)
            | Self::Levenshtein(..)
            | Self::NormalizedLevenshtein(..)
            | Self::OsaDistance(..) => "distance",

            // HTML operations
            Self::HtmlEncode(..) | Self::HtmlSanitize(..) => "html",

            // Type checking
            Self::IsAlphanum(..)
            | Self::IsAlpha(..)
            | Self::IsAscii(..)
            | Self::IsDatetime(..)
            | Self::IsDomain(..)
            | Self::IsEmail(..)
            | Self::IsHexadecimal(..)
            | Self::IsIp(..)
            | Self::IsIpv4(..)
            | Self::IsIpv6(..)
            | Self::IsLatitude(..)
            | Self::IsLongitude(..)
            | Self::IsNumeric(..)
            | Self::IsRecord(..)
            | Self::IsSemver(..)
            | Self::IsUlid(..)
            | Self::IsUrl(..)
            | Self::IsUuid(..) => "validation",

            // Semver operations
            Self::SemverCompare(..)
            | Self::SemverMajor(..)
            | Self::SemverMinor(..)
            | Self::SemverPatch(..)
            | Self::SemverIncMajor(..)
            | Self::SemverIncMinor(..)
            | Self::SemverIncPatch(..)
            | Self::SemverSetMajor(..)
            | Self::SemverSetMinor(..)
            | Self::SemverSetPatch(..) => "semver",

            // Similarity functions
            Self::SimilarityFuzzy(..) | Self::SimilarityJaro(..) | Self::SimilarityJaroWinkler(..) => "similarity",
        }
    }

    fn can_filter(&self) -> bool {
        matches!(
            self,
            // All validation functions can be used in WHERE
            Self::Contains(..)
                | Self::EndsWith(..)
                | Self::StartsWith(..)
                | Self::Matches(..)
                | Self::IsAlphanum(..)
                | Self::IsAlpha(..)
                | Self::IsAscii(..)
                | Self::IsDatetime(..)
                | Self::IsDomain(..)
                | Self::IsEmail(..)
                | Self::IsHexadecimal(..)
                | Self::IsIp(..)
                | Self::IsIpv4(..)
                | Self::IsIpv6(..)
                | Self::IsLatitude(..)
                | Self::IsLongitude(..)
                | Self::IsNumeric(..)
                | Self::IsRecord(..)
                | Self::IsSemver(..)
                | Self::IsUlid(..)
                | Self::IsUrl(..)
                | Self::IsUuid(..)
        )
    }
}
