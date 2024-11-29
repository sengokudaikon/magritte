//! Math functions for SurrealDB queries

use std::fmt::{self, Display};

use super::Callable;

/// Math function types supported by SurrealDB
///
/// math::abs()    Returns the absolute value of a number
///
/// math::acos()    Computes the arccosine (inverse cosine) of a value
///
/// math::acot()    Computes the arccotangent (inverse cotangent) of an angle
/// given in radians
///
/// math::asin()    Computes the arcsine (inverse sine) of a value
///
/// math::atan()    Computes the arctangent (inverse tangent) of a value
///
/// math::bottom()    Returns the bottom X set of numbers in a set of numbers
///
/// math::ceil()    Rounds a number up to the next largest integer
///
/// math::clamp()    Clamps a value between a specified minimum and maximum
///
/// math::cos()    Computes the cosine of an angle given in radians
///
/// math::cot()    Computes the cotangent of an angle given in radians
///
/// math::deg2rad()    Converts an angle from degrees to radians
///
/// math::e    Constant representing the base of the natural logarithm (Euler’s
/// number)
///
/// math::fixed()    Returns a number with the specified number of decimal
/// places
///
/// math::floor()    Rounds a number down to the nearest integer
///
/// math::frac_1_pi    Constant representing the fraction 1/π
///
/// math::frac_1_sqrt_2    Constant representing the fraction 1/sqrt(2)
///
/// math::frac_2_pi    Constant representing the fraction 2/π
///
/// math::frac_2_sqrt_pi    Constant representing the fraction 2/sqrt(π)
///
/// math::frac_pi_2    Constant representing the fraction π/2
///
/// math::frac_pi_3    Constant representing the fraction π/3
///
/// math::frac_pi_4    Constant representing the fraction π/4
///
/// math::frac_pi_6    Constant representing the fraction π/6
///
/// math::frac_pi_8    Constant representing the fraction π/8
///
/// math::inf    Constant representing positive infinity
///
/// math::interquartile()    Returns the interquartile of an array of numbers
///
/// math::lerp()    Linearly interpolates between two values based on a factor
///
/// math::lerpangle()    Linearly interpolates between two angles in degrees
///
/// math::ln()    Computes the natural logarithm (base e) of a value
///
/// math::ln_10    Constant representing the natural logarithm (base e) of 10
///
/// math::ln_2    Constant representing the natural logarithm (base e) of 2
///
/// math::log()    Computes the logarithm of a value with the specified base
///
/// math::log10()    Computes the base-10 logarithm of a value
///
/// math::log10_2    Constant representing the base-10 logarithm of 2
///
/// math::log10_e    Constant representing the base-10 logarithm of e, the base
/// of the natural logarithm (Euler’s number)
///
/// math::log2()    Computes the base-2 logarithm of a value
///
/// math::log2_10    Constant representing the base-2 logarithm of 10
///
/// math::log2_e    Constant representing the base-2 logarithm of e, the base of
/// the natural logarithm (Euler’s number)
///
/// math::max()    Returns the maximum number in a set of numbers
///
/// math::mean()    Returns the mean of a set of numbers
///
/// math::median()    Returns the median of a set of numbers
///
/// math::midhinge()    Returns the midhinge of a set of numbers
///
/// math::min()    Returns the minimum number in a set of numbers
///
/// math::mode()    Returns the value that occurs most often in a set of numbers
///
/// math::nearestrank()    Returns the nearest rank of an array of numbers
///
/// math::neg_inf    Constant representing negative infinity
///
/// math::percentile()    Returns the value below which a percentage of data
/// falls
///
/// math::pi    Constant representing the mathematical constant π.
///
/// math::pow()    Returns a number raised to a power
///
/// math::product()    Returns the product of a set of numbers
///
/// math::rad2deg()    Converts an angle from radians to degrees
///
/// math::round()    Rounds a number up or down to the nearest integer
///
/// math::sign()    Returns the sign of a value (-1, 0, or 1)
///
/// math::sin()    Computes the sine of an angle given in radians
///
/// math::spread()    Returns the spread of an array of numbers
///
/// math::sqrt()    Returns the square root of a number
///
/// math::sqrt_2    Constant representing the square root of 2
///
/// math::stddev()    Calculates how far a set of numbers are away from the mean
///
/// math::sum()    Returns the total sum of a set of numbers
///
/// math::tan()    Computes the tangent of an angle given in radians.
///
/// math::tau()    Represents the mathematical constant τ, which is equal to 2π
///
/// math::top()    Returns the top X set of numbers in a set of numbers
///
/// math::trimean()    The weighted average of the median and the two quartiles
///
/// math::variance()    Calculates how far a set of numbers are spread out from
/// the mean
#[derive(Debug, Clone)]
pub enum MathFunction {
    /// Returns absolute value
    Abs(String),
    /// Computes arccosine
    Acos(String),
    /// Computes arccotangent
    Acot(String),
    /// Computes arcsine
    Asin(String),
    /// Computes arctangent
    Atan(String),
    /// Returns bottom X numbers
    Bottom(String, usize),
    /// Rounds up to integer
    Ceil(String),
    /// Clamps between min/max
    Clamp(String, f64, f64),
    /// Computes cosine
    Cos(String),
    /// Computes cotangent
    Cot(String),
    /// Converts degrees to radians
    Deg2rad(String),
    /// Returns fixed decimal places
    Fixed(String, u8),
    /// Rounds down to integer
    Floor(String),
    /// Computes natural logarithm
    Ln(String),
    /// Computes logarithm with base
    Log(String, f64),
    /// Computes base-10 logarithm
    Log10(String),
    /// Computes base-2 logarithm
    Log2(String),
    /// Returns maximum value
    Max(String),
    /// Returns mean value
    Mean(String),
    /// Returns median value
    Median(String),
    /// Returns minimum value
    Min(String),
    /// Returns mode value
    Mode(String),
    /// Returns percentile value
    Percentile(String, f64),
    /// Raises to power
    Pow(String, f64),
    /// Returns product of values
    Product(String),
    /// Converts radians to degrees
    Rad2deg(String),
    /// Rounds to nearest integer
    Round(String),
    /// Returns sign (-1,0,1)
    Sign(String),
    /// Computes sine
    Sin(String),
    /// Returns square root
    Sqrt(String),
    /// Returns standard deviation
    Stddev(String),
    /// Returns sum of values
    Sum(String),
    /// Computes tangent
    Tan(String),
    /// Returns top X numbers
    Top(String, usize),
    /// Returns variance
    Variance(String),
    /// Returns the interquartile of an array of numbers
    Interquartile(String),
    /// Linearly interpolates between two values based on a factor
    Lerp(String, String, String), // value1, value2, factor
    /// Linearly interpolates between two angles in degrees
    LerpAngle(String, String, String), // angle1, angle2, factor
    /// Returns the midhinge of a set of numbers
    Midhinge(String),
    /// Returns the spread of an array of numbers
    Spread(String),
    /// Returns the trimean of an array of numbers
    Trimean(String),
    /// Mathematical constant e (Euler's number)
    E,
    /// Mathematical constant π
    Pi,
    /// Mathematical constant τ (2π)
    Tau,
    /// Mathematical constant 1/π
    Frac1Pi,
    /// Mathematical constant 2/π
    Frac2Pi,
    /// Mathematical constant 1/sqrt(2)
    Frac1Sqrt2,
    /// Mathematical constant 2/sqrt(π)
    Frac2SqrtPi,
    /// Mathematical constant π/2
    FracPi2,
    /// Mathematical constant π/3
    FracPi3,
    /// Mathematical constant π/4
    FracPi4,
    /// Mathematical constant π/6
    FracPi6,
    /// Mathematical constant π/8
    FracPi8,
    /// Mathematical constant ln(2)
    Ln2,
    /// Mathematical constant ln(10)
    Ln10,
    /// Mathematical constant log₂(e)
    Log2E,
    /// Mathematical constant log₁₀(e)
    Log10E,
    /// Mathematical constant log₂(10)
    Log210,
    /// Mathematical constant log₁₀(2)
    Log102,
    /// Mathematical constant sqrt(2)
    Sqrt2,
    /// Mathematical constant infinity
    Inf,
}

impl Display for MathFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Abs(val) => write!(f, "math::abs({})", val),
            Self::Acos(val) => write!(f, "math::acos({})", val),
            Self::Acot(val) => write!(f, "math::acot({})", val),
            Self::Asin(val) => write!(f, "math::asin({})", val),
            Self::Atan(val) => write!(f, "math::atan({})", val),
            Self::Bottom(val, places) => write!(f, "math::bottom({}, {})", val, places),
            Self::Ceil(val) => write!(f, "math::ceil({})", val),
            Self::Clamp(val, min, max) => write!(f, "math::clamp({}, {}, {})", val, min, max),
            Self::Cos(val) => write!(f, "math::cos({})", val),
            Self::Cot(val) => write!(f, "math::cot({})", val),
            Self::Deg2rad(val) => write!(f, "math::deg2rad({})", val),
            Self::Fixed(val, places) => write!(f, "math::fixed({}, {})", val, places),
            Self::Floor(val) => write!(f, "math::floor({})", val),
            Self::Ln(val) => write!(f, "math::ln({})", val),
            Self::Log(val, base) => write!(f, "math::log({}, {})", val, base),
            Self::Log10(val) => write!(f, "math::log10({})", val),
            Self::Log2(val) => write!(f, "math::log2({})", val),
            Self::Max(val) => write!(f, "math::max({})", val),
            Self::Mean(val) => write!(f, "math::mean({})", val),
            Self::Median(val) => write!(f, "math::median({})", val),
            Self::Min(val) => write!(f, "math::min({})", val),
            Self::Mode(val) => write!(f, "math::mode({})", val),
            Self::Percentile(val, percentile) => write!(f, "math::percentile({}, {})", val, percentile),
            Self::Pow(val, power) => write!(f, "math::pow({}, {})", val, power),
            Self::Product(val) => write!(f, "math::product({})", val),
            Self::Rad2deg(val) => write!(f, "math::rad2deg({})", val),
            Self::Round(val) => write!(f, "math::round({})", val),
            Self::Sign(val) => write!(f, "math::sign({})", val),
            Self::Sin(val) => write!(f, "math::sin({})", val),
            Self::Sqrt(val) => write!(f, "math::sqrt({})", val),
            Self::Stddev(val) => write!(f, "math::stddev({})", val),
            Self::Sum(val) => write!(f, "math::sum({})", val),
            Self::Tan(val) => write!(f, "math::tan({})", val),
            Self::Top(val, places) => write!(f, "math::top({}, {})", val, places),
            Self::Variance(val) => write!(f, "math::variance({})", val),
            Self::Interquartile(val) => write!(f, "math::interquartile({})", val),
            Self::Lerp(v1, v2, factor) => write!(f, "math::lerp({}, {}, {})", v1, v2, factor),
            Self::LerpAngle(a1, a2, factor) => write!(f, "math::lerpangle({}, {}, {})", a1, a2, factor),
            Self::Midhinge(val) => write!(f, "math::midhinge({})", val),
            Self::Spread(val) => write!(f, "math::spread({})", val),
            Self::Trimean(val) => write!(f, "math::trimean({})", val),
            Self::E => write!(f, "math::e"),
            Self::Pi => write!(f, "math::pi"),
            Self::Tau => write!(f, "math::tau"),
            Self::Frac1Pi => write!(f, "math::frac_1_pi"),
            Self::Frac2Pi => write!(f, "math::frac_2_pi"),
            Self::Frac1Sqrt2 => write!(f, "math::frac_1_sqrt_2"),
            Self::Frac2SqrtPi => write!(f, "math::frac_2_sqrt_pi"),
            Self::FracPi2 => write!(f, "math::frac_pi_2"),
            Self::FracPi3 => write!(f, "math::frac_pi_3"),
            Self::FracPi4 => write!(f, "math::frac_pi_4"),
            Self::FracPi6 => write!(f, "math::frac_pi_6"),
            Self::FracPi8 => write!(f, "math::frac_pi_8"),
            Self::Ln2 => write!(f, "math::ln_2"),
            Self::Ln10 => write!(f, "math::ln_10"),
            Self::Log2E => write!(f, "math::log2_e"),
            Self::Log10E => write!(f, "math::log10_e"),
            Self::Log210 => write!(f, "math::log2_10"),
            Self::Log102 => write!(f, "math::log10_2"),
            Self::Sqrt2 => write!(f, "math::sqrt_2"),
            Self::Inf => write!(f, "math::inf"),
        }
    }
}

impl Callable for MathFunction {
    fn namespace() -> &'static str { "math" }

    fn category(&self) -> &'static str {
        match self {
            // Constants
            Self::E
            | Self::Pi
            | Self::Tau
            | Self::Frac1Pi
            | Self::Frac2Pi
            | Self::Frac1Sqrt2
            | Self::Frac2SqrtPi
            | Self::FracPi2
            | Self::FracPi3
            | Self::FracPi4
            | Self::FracPi6
            | Self::FracPi8
            | Self::Ln2
            | Self::Ln10
            | Self::Log2E
            | Self::Log10E
            | Self::Log210
            | Self::Log102
            | Self::Sqrt2
            | Self::Inf => "constant",

            // Statistical functions
            Self::Interquartile(..)
            | Self::Midhinge(..)
            | Self::Spread(..)
            | Self::Trimean(..)
            | Self::Mean(..)
            | Self::Median(..)
            | Self::Mode(..)
            | Self::Stddev(..)
            | Self::Variance(..) => "statistical",

            // Interpolation functions
            Self::Lerp(..) | Self::LerpAngle(..) => "interpolation",

            // Basic math functions
            _ => "basic",
        }
    }

    fn can_filter(&self) -> bool {
        false // Math functions return numeric values, not boolean
    }
}
