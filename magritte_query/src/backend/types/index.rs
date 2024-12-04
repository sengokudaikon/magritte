use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::vector_search::{VectorDistance, VectorType};

#[derive(Debug, Clone, PartialEq)]
pub enum IndexSpecifics {
    None,
    Search {
        analyzer: Option<String>,
        bm25: Option<(f32, f32)>, // (k1, b)
        highlights: bool,
    },
    MTREE {
        dimension: i32,
        vector_type: Option<VectorType>,
        dist: VectorDistance,
        capacity: Option<i64>,
    },
    HNSW {
        dimension: i32,
        vector_type: Option<VectorType>,
        dist: VectorDistance,
        efc: Option<i32>,
        m: Option<i32>,
    },
}
impl From<&String> for IndexSpecifics {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "" => IndexSpecifics::None,
            s if s.starts_with("SEARCH") => {
                let mut parts = s.split_whitespace();
                parts.next(); // Skip "SEARCH"
                let analyzer = parts.find(|&part| part.starts_with("ANALYZER")).map(|s| s[9..].to_string());
                let bm25 = parts.find(|&part| part.starts_with("BM25")).map(|s| {
                    let nums: Vec<f32> = s[5..s.len() - 1].split(',').map(|n| n.trim().parse().unwrap()).collect();
                    (nums[0], nums[1])
                });
                let highlights = parts.any(|part| part == "HIGHLIGHTS");
                IndexSpecifics::Search { analyzer, bm25, highlights }
            },
            s if s.starts_with("MTREE") => {
                let mut parts = s.split_whitespace();
                parts.next(); // Skip "MTREE"
                let dimension = parts.next().unwrap().parse().unwrap();
                let vector_type = parts.next().and_then(|s| VectorType::from_str(s).ok());
                let dist = VectorDistance::from_str(parts.next().unwrap()).unwrap();
                let capacity = parts.next().and_then(|s| s.parse().ok());
                IndexSpecifics::MTREE { dimension, vector_type, dist, capacity }
            },
            s if s.starts_with("HNSW") => {
                let mut parts = s.split_whitespace();
                parts.next(); // Skip "HNSW"
                let dimension = parts.next().unwrap().parse().unwrap();
                let vector_type = parts.next().and_then(|s| VectorType::from_str(s).ok());
                let dist = VectorDistance::from_str(parts.next().unwrap()).unwrap();
                let efc = parts.next().and_then(|s| s.parse().ok());
                let m = parts.next().and_then(|s| s.parse().ok());
                IndexSpecifics::HNSW { dimension, vector_type, dist, efc, m }
            },
            _ => IndexSpecifics::None, // Default case if pattern does not match
        }
    }
}
impl Display for IndexSpecifics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Search { analyzer, bm25, highlights } => {
                let bm25 = if let Some(bm25) = bm25 {
                    format!("BM25({}, {})", bm25.0, bm25.1)
                } else {
                    "".to_string()
                };
                let analyzer = if let Some(analyzer) = analyzer {
                    format!("ANALYZER {}", analyzer)
                } else {
                    "".to_string()
                };
                let highlights = if *highlights { " HIGHLIGHTS" } else { "" };
                write!(f, "SEARCH {} {} {}", &analyzer, &bm25, highlights)
            },
            Self::MTREE { dimension, vector_type, dist, capacity } => {
                let vector_type = if let Some(vector_type) = vector_type {
                    format!(" TYPE {}", vector_type)
                } else {
                    "".to_string()
                };
                let dist = format!("DIST {}", dist);
                let capacity = if let Some(capacity) = capacity {
                    format!(" CAPACITY {}", capacity)
                } else {
                    "".to_string()
                };
                write!(f, "MTREE DIMENSION {}{}{}{}", dimension, vector_type, dist, capacity)

            },
            Self::HNSW {dimension, vector_type, dist, efc, m } => {
                let vector_type = if let Some(vector_type) = vector_type {
                    format!(" TYPE {}", vector_type)
                } else {
                    "".to_string()
                };
                let dist = format!("DIST {}", dist);
                let efc = if let Some(efc) = efc {
                    format!(" EFC {}", efc)
                } else {
                    "".to_string()
                };
                let m = if let Some(m) = m {
                    format!(" M {}", m)
                } else {
                    "".to_string()
                };
                write!(f, "HNSW DIMENSION {}{}{}{}{}", dimension, vector_type, dist, efc, m)
            }
        }
    }
}
