//! Geo functions for SurrealDB queries
//!
//! These functions can be used when working with and manipulating geographic
//! data.

use std::fmt::{self, Display};

use super::Callable;

/// Geo function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum GeoFunction {
    /// Returns the area of a geometry in square meters
    Area(String),
    /// Returns the bearing between two points in degrees
    Bearing(String, String),
    /// Returns the centroid point of a geometry
    Centroid(String),
    /// Returns the distance between two points in meters
    Distance(String, String),
    /// Returns whether a geometry contains another geometry
    Contains(String, String),
    /// Returns whether a geometry intersects another geometry
    Intersects(String, String),
    /// Returns whether a geometry is valid
    IsValid(String),
    /// Returns the length of a line in meters
    Length(String),
    /// Returns a point at a specific latitude and longitude
    Point(f64, f64), // lat, lon
    /// Returns a polygon from an array of points
    Polygon(Vec<(f64, f64)>),
}

impl Display for GeoFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Area(geom) => write!(f, "geo::area({})", geom),
            Self::Bearing(p1, p2) => write!(f, "geo::bearing({}, {})", p1, p2),
            Self::Centroid(geom) => write!(f, "geo::centroid({})", geom),
            Self::Distance(p1, p2) => write!(f, "geo::distance({}, {})", p1, p2),
            Self::Contains(g1, g2) => write!(f, "geo::contains({}, {})", g1, g2),
            Self::Intersects(g1, g2) => write!(f, "geo::intersects({}, {})", g1, g2),
            Self::IsValid(geom) => write!(f, "geo::is::valid({})", geom),
            Self::Length(line) => write!(f, "geo::length({})", line),
            Self::Point(lat, lon) => write!(f, "geo::point({}, {})", lat, lon),
            Self::Polygon(points) => {
                let points_str = points
                    .iter()
                    .map(|(lat, lon)| format!("[{}, {}]", lat, lon))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "geo::polygon([{}])", points_str)
            }
        }
    }
}

impl Callable for GeoFunction {
    fn namespace() -> &'static str {
        "geo"
    }

    fn category(&self) -> &'static str {
        match self {
            // Measurement functions
            Self::Area(..) | Self::Distance(..) | Self::Length(..) => "measurement",

            // Analysis functions
            Self::Bearing(..) | Self::Centroid(..) => "analysis",

            // Validation functions
            Self::Contains(..) | Self::Intersects(..) | Self::IsValid(..) => "validation",

            // Construction functions
            Self::Point(..) | Self::Polygon(..) => "construction",
        }
    }

    fn can_filter(&self) -> bool {
        matches!(
            self,
            // Only validation functions can be used in WHERE
            Self::Contains(..) | Self::Intersects(..) | Self::IsValid(..)
        )
    }
}
