//! Vector functions for SurrealDB queries
//!
//! A collection of essential vector operations that provide foundational
//! functionality for numerical computation, machine learning, and data
//! analysis.

use std::fmt::{self, Display};

use super::Callable;

/// Vector function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum VectorFunction {
    // Basic vector operations
    /// Performs element-wise addition of two vectors
    Add(String, String),
    /// Computes the angle between two vectors
    Angle(String, String),
    /// Computes the cross product of two vectors
    Cross(String, String),
    /// Performs element-wise division between two vectors
    Divide(String, String),
    /// Computes the dot product of two vectors
    Dot(String, String),
    /// Computes the magnitude (or length) of a vector
    Magnitude(String),
    /// Performs element-wise multiplication of two vectors
    Multiply(String, String),
    /// Computes the normalization of a vector
    Normalize(String),
    /// Computes the projection of one vector onto another
    Project(String, String),
    /// Multiplies each item in a vector by a number
    Scale(String, f64),
    /// Performs element-wise subtraction between two vectors
    Subtract(String, String),

    // Distance functions
    /// Computes the Chebyshev distance
    DistanceChebyshev(String, String),
    /// Computes the Euclidean distance between two vectors
    DistanceEuclidean(String, String),
    /// Computes the Hamming distance between two vectors
    DistanceHamming(String, String),
    /// Returns the distance computed during the query
    DistanceKnn,
    /// Computes the Manhattan distance between two vectors
    DistanceManhattan(String, String),
    /// Computes the Minkowski distance between two vectors
    DistanceMinkowski(String, String, f64), // vec1, vec2, p

    // Similarity functions
    /// Computes the Cosine similarity between two vectors
    SimilarityCosine(String, String),
    /// Computes the Jaccard similarity between two vectors
    SimilarityJaccard(String, String),
    /// Computes the Pearson correlation coefficient between two vectors
    SimilarityPearson(String, String),
}

impl Display for VectorFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Basic vector operations
            Self::Add(v1, v2) => write!(f, "vector::add({}, {})", v1, v2),
            Self::Angle(v1, v2) => write!(f, "vector::angle({}, {})", v1, v2),
            Self::Cross(v1, v2) => write!(f, "vector::cross({}, {})", v1, v2),
            Self::Divide(v1, v2) => write!(f, "vector::divide({}, {})", v1, v2),
            Self::Dot(v1, v2) => write!(f, "vector::dot({}, {})", v1, v2),
            Self::Magnitude(v) => write!(f, "vector::magnitude({})", v),
            Self::Multiply(v1, v2) => write!(f, "vector::multiply({}, {})", v1, v2),
            Self::Normalize(v) => write!(f, "vector::normalize({})", v),
            Self::Project(v1, v2) => write!(f, "vector::project({}, {})", v1, v2),
            Self::Scale(v, n) => write!(f, "vector::scale({}, {})", v, n),
            Self::Subtract(v1, v2) => write!(f, "vector::subtract({}, {})", v1, v2),

            // Distance functions
            Self::DistanceChebyshev(v1, v2) => {
                write!(f, "vector::distance::chebyshev({}, {})", v1, v2)
            }
            Self::DistanceEuclidean(v1, v2) => {
                write!(f, "vector::distance::euclidean({}, {})", v1, v2)
            }
            Self::DistanceHamming(v1, v2) => write!(f, "vector::distance::hamming({}, {})", v1, v2),
            Self::DistanceKnn => write!(f, "vector::distance::knn()"),
            Self::DistanceManhattan(v1, v2) => {
                write!(f, "vector::distance::manhattan({}, {})", v1, v2)
            }
            Self::DistanceMinkowski(v1, v2, p) => {
                write!(f, "vector::distance::minkowski({}, {}, {})", v1, v2, p)
            }

            // Similarity functions
            Self::SimilarityCosine(v1, v2) => {
                write!(f, "vector::similarity::cosine({}, {})", v1, v2)
            }
            Self::SimilarityJaccard(v1, v2) => {
                write!(f, "vector::similarity::jaccard({}, {})", v1, v2)
            }
            Self::SimilarityPearson(v1, v2) => {
                write!(f, "vector::similarity::pearson({}, {})", v1, v2)
            }
        }
    }
}

impl Callable for VectorFunction {
    fn namespace() -> &'static str {
        "vector"
    }

    fn category(&self) -> &'static str {
        match self {
            // Basic vector operations
            Self::Add(..)
            | Self::Subtract(..)
            | Self::Multiply(..)
            | Self::Divide(..)
            | Self::Scale(..) => "arithmetic",

            // Vector products and angles
            Self::Dot(..) | Self::Cross(..) | Self::Angle(..) => "product",

            // Vector properties
            Self::Magnitude(..) | Self::Normalize(..) => "property",

            // Vector projections
            Self::Project(..) => "projection",

            // Distance metrics
            Self::DistanceChebyshev(..)
            | Self::DistanceEuclidean(..)
            | Self::DistanceHamming(..)
            | Self::DistanceKnn
            | Self::DistanceManhattan(..)
            | Self::DistanceMinkowski(..) => "distance",

            // Similarity metrics
            Self::SimilarityCosine(..)
            | Self::SimilarityJaccard(..)
            | Self::SimilarityPearson(..) => "similarity",
        }
    }

    fn can_filter(&self) -> bool {
        false // Vector functions return vectors or numeric values, not boolean
    }
}
