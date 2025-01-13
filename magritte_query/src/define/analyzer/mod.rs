//! Analyzer definition functionality for SurrealDB.
//!
//! This module provides functionality to define text analyzers in SurrealDB. Analyzers are used
//! for text processing and searching, defined by a set of tokenizers and filters.
//!
//! See [SurrealDB Analyzer Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer)
//! for more details.
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Create a basic analyzer for English text
//! let analyzer = Define::analyzer()
//!     .name("english")
//!     .tokenizers(vec![Tokenizer::Class, Tokenizer::Camel])
//!     .filters(vec![
//!         Filter::Lowercase,
//!         Filter::Ascii,
//!         Filter::Snowball("english".to_string())
//!     ])
//!     .comment("English text analyzer")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root, namespace, or database user
//! - Selected namespace and database
//!
//! For more information, see [SurrealDB Requirements](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#requirements)

use crate::database::{QueryType, SurrealDB};
use anyhow::{anyhow, bail};
use std::fmt::Display;
use tracing::{error, info};

/// List of supported languages for the Snowball filter.
///
/// These languages are officially supported by SurrealDB's Snowball stemming algorithm.
/// See [Snowball Filter Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#snowballlanguage)
pub const SUPPORTED_SNOWBALL_LANGUAGES: &[&str] = &[
    "arabic",
    "danish",
    "dutch",
    "english",
    "french",
    "german",
    "greek",
    "hungarian",
    "italian",
    "norwegian",
    "portuguese",
    "romanian",
    "russian",
    "spanish",
    "swedish",
    "tamil",
    "turkish",
];

/// Supported tokenizer types for analyzers.
///
/// Tokenizers are responsible for breaking down text into individual tokens based on specific rules.
/// See [SurrealDB Tokenizers Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#tokenizers)
#[derive(Clone, Debug)]
pub enum Tokenizer {
    /// Breaks text on whitespace (space, tab, newline).
    ///
    /// Example:
    /// ```text
    /// "hello world" -> ["hello", "world"]
    /// ```
    /// See [Blank Tokenizer](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#blank)
    Blank,

    /// Breaks on camelCase and PascalCase transitions.
    ///
    /// Example:
    /// ```text
    /// "helloWorld" -> ["hello", "World"]
    /// ```
    /// See [Camel Tokenizer](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#camel)
    Camel,

    /// Breaks on Unicode class changes (digit, letter, punctuation, blank).
    ///
    /// Example:
    /// ```text
    /// "123abc!XYZ" -> ["123", "abc", "!", "XYZ"]
    /// ```
    /// See [Class Tokenizer](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#class)
    Class,

    /// Breaks on punctuation characters.
    ///
    /// Example:
    /// ```text
    /// "Hello, World!" -> ["Hello", ",", "World", "!"]
    /// ```
    /// See [Punct Tokenizer](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#punct)
    Punct,
}

impl Display for Tokenizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tokenizer::Blank => write!(f, "blank"),
            Tokenizer::Camel => write!(f, "camel"),
            Tokenizer::Class => write!(f, "class"),
            Tokenizer::Punct => write!(f, "punct"),
        }
    }
}

/// Supported filter types for analyzers.
///
/// Filters transform tokens after they've been created by tokenizers.
/// See [SurrealDB Filters Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#filters)
#[derive(Clone, Debug)]
pub enum Filter {
    /// Removes diacritical marks and converts to ASCII.
    ///
    /// Example:
    /// ```text
    /// "résumé café" -> ["resume", "cafe"]
    /// ```
    /// See [ASCII Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#ascii)
    Ascii,

    /// Converts text to lowercase.
    ///
    /// Example:
    /// ```text
    /// "Hello World" -> ["hello", "world"]
    /// ```
    /// See [Lowercase Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#lowercase)
    Lowercase,

    /// Converts text to uppercase.
    ///
    /// Example:
    /// ```text
    /// "Hello World" -> ["HELLO", "WORLD"]
    /// ```
    /// See [Uppercase Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#uppercase)
    Uppercase,

    /// Creates edge n-grams from min to max length.
    ///
    /// Example with min=1, max=3:
    /// ```text
    /// "apple banana" -> ["a", "ap", "app", "b", "ba", "ban"]
    /// ```
    /// See [EdgeNGram Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#edgengramminmax)
    EdgeNGram(u32, u32),

    /// Maps terms using a custom dictionary file.
    ///
    /// Used for lemmatization and custom term mapping.
    /// See [Mapper Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#mapperpath)
    Mapper(String),

    /// Creates n-grams from min to max length.
    ///
    /// Example with min=1, max=3:
    /// ```text
    /// "apple" -> ["a", "ap", "app", "p", "pp", "ppl", ...]
    /// ```
    /// See [NGram Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#ngramminmax)
    NGram(u32, u32),

    /// Applies Snowball stemming for the specified language.
    ///
    /// Example:
    /// ```text
    /// "Looking at some running cats" -> ["look", "at", "some", "run", "cat"]
    /// ```
    /// See [Snowball Filter](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer#snowballlanguage)
    Snowball(String),
}

impl Filter {
    /// Creates a new Snowball filter with language validation.
    ///
    /// # Arguments
    ///
    /// * `language` - One of the supported Snowball stemming languages
    ///
    /// # Returns
    ///
    /// * `Ok(Filter)` if the language is supported
    /// * `Err` with an error message listing supported languages if not
    pub fn snowball(language: impl Into<String>) -> anyhow::Result<Self> {
        let lang = language.into().to_lowercase();
        if SUPPORTED_SNOWBALL_LANGUAGES.contains(&lang.as_str()) {
            Ok(Filter::Snowball(lang))
        } else {
            bail!(
                "Unsupported Snowball language: {}. Supported languages are: {}",
                lang,
                SUPPORTED_SNOWBALL_LANGUAGES.join(", ")
            )
        }
    }

    /// Creates a new EdgeNGram filter with validation.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum length of generated n-grams (must be > 0)
    /// * `max` - Maximum length of generated n-grams (must be >= min)
    ///
    /// # Returns
    ///
    /// * `Ok(Filter)` if the parameters are valid
    /// * `Err` with an error message if validation fails
    pub fn edge_ngram(min: u32, max: u32) -> anyhow::Result<Self> {
        if min > max {
            bail!(
                "EdgeNGram min length ({}) cannot be greater than max length ({})",
                min,
                max
            );
        }
        if min == 0 {
            bail!("EdgeNGram min length must be greater than 0");
        }
        Ok(Filter::EdgeNGram(min, max))
    }

    /// Creates a new NGram filter with validation.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum length of generated n-grams (must be > 0)
    /// * `max` - Maximum length of generated n-grams (must be >= min)
    ///
    /// # Returns
    ///
    /// * `Ok(Filter)` if the parameters are valid
    /// * `Err` with an error message if validation fails
    pub fn ngram(min: u32, max: u32) -> anyhow::Result<Self> {
        if min > max {
            bail!(
                "NGram min length ({}) cannot be greater than max length ({})",
                min,
                max
            );
        }
        if min == 0 {
            bail!("NGram min length must be greater than 0");
        }
        Ok(Filter::NGram(min, max))
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Ascii => write!(f, "ascii"),
            Filter::Lowercase => write!(f, "lowercase"),
            Filter::Uppercase => write!(f, "uppercase"),
            Filter::EdgeNGram(min, max) => write!(f, "edgengram({},{})", min, max),
            Filter::Mapper(path) => write!(f, "mapper('{}')", path),
            Filter::NGram(min, max) => write!(f, "ngram({},{})", min, max),
            Filter::Snowball(lang) => write!(f, "snowball({})", lang),
        }
    }
}

/// Statement for defining text analyzers in SurrealDB.
///
/// An analyzer processes text for searching and indexing through a combination of
/// tokenizers and filters. Tokenizers break text into tokens, and filters transform
/// these tokens.
///
/// See [DEFINE ANALYZER Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/analyzer)
///
/// # Example
///
/// ```rust
/// use magritte_query::define::*;
///
/// // Create an analyzer for autocomplete functionality
/// let analyzer = Define::analyzer()
///     .name("autocomplete")
///     .filter(Filter::Lowercase)
///     .filter(Filter::EdgeNGram(2, 10))
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug, Default)]
pub struct DefineAnalyzerStatement {
    pub(crate) name: Option<String>,
    pub(crate) tokenizers: Vec<Tokenizer>,
    pub(crate) filters: Vec<Filter>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineAnalyzerStatement {
    /// Creates a new empty analyzer statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the name of the analyzer
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Adds a single tokenizer to the analyzer
    pub fn tokenizer(mut self, tokenizer: Tokenizer) -> Self {
        self.tokenizers.push(tokenizer);
        self
    }

    /// Adds multiple tokenizers to the analyzer
    pub fn tokenizers(mut self, tokenizers: Vec<Tokenizer>) -> Self {
        self.tokenizers.extend(tokenizers);
        self
    }

    /// Adds a single filter to the analyzer
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Adds multiple filters to the analyzer
    pub fn filters(mut self, filters: Vec<Filter>) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Sets the OVERWRITE clause
    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Sets the IF NOT EXISTS clause
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Adds a comment to the analyzer definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the analyzer definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE ANALYZER ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name);
        } else {
            bail!("Analyzer name is required");
        }

        if !self.tokenizers.is_empty() {
            stmt.push_str(" TOKENIZERS ");
            stmt.push_str(
                &self
                    .tokenizers
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        if !self.filters.is_empty() {
            stmt.push_str(" FILTERS ");
            stmt.push_str(
                &self
                    .filters
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the analyzer definition statement on the database
    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        conn.execute(self.build()?, vec![], QueryType::Schema, None).await
    }
}

impl Display for DefineAnalyzerStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analyzer() {
        let stmt = DefineAnalyzerStatement::new()
            .name("example")
            .tokenizer(Tokenizer::Blank)
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE ANALYZER example TOKENIZERS blank;");
    }

    #[test]
    fn test_complex_analyzer() {
        let stmt = DefineAnalyzerStatement::new()
            .name("english")
            .tokenizers(vec![Tokenizer::Class, Tokenizer::Camel])
            .filters(vec![
                Filter::Lowercase,
                Filter::Ascii,
                Filter::Snowball("english".to_string()),
            ])
            .comment("English text analyzer")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE ANALYZER english TOKENIZERS class,camel FILTERS lowercase,ascii,snowball(english) COMMENT \"English text analyzer\";");
    }

    #[test]
    fn test_edge_ngram_analyzer() {
        let stmt = DefineAnalyzerStatement::new()
            .name("autocomplete")
            .filter(Filter::Lowercase)
            .filter(Filter::EdgeNGram(2, 10))
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE ANALYZER autocomplete FILTERS lowercase,edgengram(2,10);"
        );
    }

    #[test]
    fn test_snowball_language_validation() {
        assert!(Filter::snowball("english").is_ok());
        assert!(Filter::snowball("ENGLISH").is_ok());
        assert!(Filter::snowball("invalid").is_err());
    }

    #[test]
    fn test_ngram_validation() {
        assert!(Filter::ngram(1, 3).is_ok());
        assert!(Filter::ngram(3, 1).is_err());
        assert!(Filter::ngram(0, 3).is_err());
    }

    #[test]
    fn test_edge_ngram_validation() {
        assert!(Filter::edge_ngram(1, 3).is_ok());
        assert!(Filter::edge_ngram(3, 1).is_err());
        assert!(Filter::edge_ngram(0, 3).is_err());
    }
}
