//! Search functions for SurrealDB queries
//!
//! These functions are used in conjunction with the @@ operator (the 'matches'
//! operator) to either collect the relevance score or highlight the searched
//! keywords within the content.

use std::fmt::{self, Display};

use super::Callable;

/// Search function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum SearchFunction {
    /// Returns the output of a defined search analyzer
    Analyze(String, String), // analyzer, string
    /// Highlights the matching keywords
    Highlight(String, String, usize, Option<bool>), // prefix, suffix, predicate_ref, whole_term
    /// Returns the position of the matching keywords
    Offsets(usize, Option<bool>), // predicate_ref, whole_term
    /// Returns the relevance score
    Score(usize), // predicate_ref
}

impl Display for SearchFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Analyze(analyzer, text) => write!(f, "search::analyze({}, {})", analyzer, text),
            Self::Highlight(prefix, suffix, pred_ref, whole_term) => {
                if let Some(whole) = whole_term {
                    write!(f, "search::highlight({}, {}, {}, {})", prefix, suffix, pred_ref, whole)
                }
                else {
                    write!(f, "search::highlight({}, {}, {})", prefix, suffix, pred_ref)
                }
            }
            Self::Offsets(pred_ref, whole_term) => {
                if let Some(whole) = whole_term {
                    write!(f, "search::offsets({}, {})", pred_ref, whole)
                }
                else {
                    write!(f, "search::offsets({})", pred_ref)
                }
            }
            Self::Score(pred_ref) => write!(f, "search::score({})", pred_ref),
        }
    }
}

impl Callable for SearchFunction {
    fn namespace() -> &'static str { "search" }

    fn category(&self) -> &'static str {
        match self {
            Self::Analyze(..) => "analysis",
            Self::Highlight(..) => "highlighting",
            Self::Offsets(..) => "position",
            Self::Score(..) => "relevance",
        }
    }

    fn can_filter(&self) -> bool {
        false // Search functions return analysis results, not boolean
              // conditions
    }
}
