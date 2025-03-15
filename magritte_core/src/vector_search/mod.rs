//! Vector search functionality for SurrealDB queries
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
pub trait Indexable {
    fn with_index(&self) -> &Option<Vec<String>>;
    fn with_index_mut(&mut self) -> &mut Option<Vec<String>>;
}
/// Vector search operators
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum VectorDistance {
    /// Cosine similarity
    #[default]
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Manhattan distance
    Manhattan,
    /// Hamming distance
    Hamming,
    /// Chebyshev distance
    Chebyshev,
    /// Minkowski distance
    Minkowski(f64),
}
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Hash)]
#[non_exhaustive]
pub enum VectorType {
    #[default]
    F64,
    F32,
    I64,
    I32,
    I16,
}
impl FromStr for VectorType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(VectorType::from(s.to_string()))
    }
}
impl Display for VectorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::F64 => write!(f, "f64"),
            Self::F32 => write!(f, "f32"),
            Self::I64 => write!(f, "i64"),
            Self::I32 => write!(f, "i32"),
            Self::I16 => write!(f, "i16"),
        }
    }
}

impl From<String> for VectorType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "f64" => Self::F64,
            "f32" => Self::F32,
            "i64" => Self::I64,
            "i32" => Self::I32,
            "i16" => Self::I16,
            _ => panic!("Invalid vector type: {}", value),
        }
    }
}

impl Display for VectorDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cosine => write!(f, "cosine"),
            Self::Euclidean => write!(f, "euclidean"),
            Self::Manhattan => write!(f, "manhattan"),
            Self::Hamming => write!(f, "hamming"),
            Self::Chebyshev => write!(f, "chebyshev"),
            Self::Minkowski(p) => write!(f, "minkowski({})", p),
        }
    }
}
impl FromStr for VectorDistance {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(VectorDistance::from(s.to_string()))
    }
}
impl From<String> for VectorDistance {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "cosine" => Self::Cosine,
            "euclidean" => Self::Euclidean,
            "manhattan" => Self::Manhattan,
            "hamming" => Self::Hamming,
            "chebyshev" => Self::Chebyshev,
            _ => Self::Minkowski(value.parse().unwrap()),
        }
    }
}

/// Vector search conditions
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum VectorCondition {
    /// Similarity search with optional threshold
    Similarity {
        field: String,
        vector: Vec<f32>,
        distance: VectorDistance,
        threshold: Option<f32>,
    },
    /// K-nearest neighbors search
    Nearest {
        field: String,
        vector: Vec<f32>,
        k: usize,
        distance: VectorDistance,
    },
    /// Batch similarity search
    BatchSimilarity {
        conditions: Vec<(String, Vec<f32>, VectorDistance, Option<f32>)>,
    },
    /// Batch nearest neighbors search
    BatchNearest {
        conditions: Vec<(String, Vec<f32>, VectorDistance, usize)>,
    },
}
