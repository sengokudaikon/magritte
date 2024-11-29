//! Array functions for SurrealDB queries

use std::fmt::{self, Display};

use super::Callable;

/// Array function types supported by SurrealDB
#[derive(Debug, Clone)]
pub enum ArrayFunction {
    /// Adds an item to an array if it doesn't exist
    Add(String, String),
    /// Checks whether all array values are truthy or match a condition
    All(String, Option<String>),
    /// Checks whether any array value is truthy or match a condition
    Any(String, Option<String>),
    /// Returns value for X index
    At(String, isize),
    /// Appends an item to the end of an array
    Append(String, String),
    /// Performs AND bitwise operations
    BooleanAnd(String, String),
    /// Performs OR bitwise operations
    BooleanOr(String, String),
    /// Performs XOR bitwise operations
    BooleanXor(String, String),
    /// Performs NOT bitwise operations
    BooleanNot(String),
    /// Combines all values from two arrays together
    Combine(Vec<String>),
    /// Returns the complement of two arrays
    Complement(String, String),
    /// Returns array split into multiple arrays of X size
    Clump(String, usize),
    /// Returns the merged values from two arrays
    Concat(Vec<String>),
    /// Returns the difference between two arrays
    Difference(String, String),
    /// Returns the unique items in an array
    Distinct(String),
    /// Fills an array with a value, optionally between start and end
    Fill(String, String, Option<(usize, usize)>),
    /// Filters out values that don't match a pattern or closure
    Filter(String, String),
    /// Returns indexes of matching values
    FilterIndex(String, String),
    /// Returns first matching value
    Find(String, String),
    /// Returns index of first matching value
    FindIndex(String, String),
    /// Returns first item in array
    First(String),
    /// Flattens multiple arrays into a single array
    Flatten(String),
    /// Applies operation on initial value plus array elements
    Fold(String, String, String),
    /// Groups and returns unique items
    Group(String),
    /// Inserts an item at position
    Insert(String, usize, String),
    /// Returns intersecting values
    Intersect(Vec<String>),
    /// Checks if array is empty
    IsEmpty(String),
    /// Returns concatenated value with separator
    Join(String, String),
    /// Returns last item
    Last(String),
    /// Returns array length
    Len(String),
    /// Performs AND logical operations
    LogicalAnd(String, String),
    /// Performs OR logical operations
    LogicalOr(String, String),
    /// Performs XOR logical operations
    LogicalXor(String, String),
    /// Maps array through closure
    Map(String, String),
    /// Returns maximum item
    Max(String),
    /// Returns array of booleans indicating matches
    Matches(String, String),
    /// Returns minimum item
    Min(String),
    /// Returns last item
    Pop(String),
    /// Prepends item to start
    Prepend(String, String),
    /// Appends item to end
    Push(String, String),
    /// Creates number array from range
    Range(i64, i64),
    /// Reduces array with closure
    Reduce(String, String),
    /// Removes item at position
    Remove(String, usize),
    /// Creates array of size with value
    Repeat(String, usize),
    /// Reverses array order
    Reverse(String),
    /// Randomly shuffles array
    Shuffle(String),
    /// Returns array slice
    Slice(String, usize, usize),
    /// Sorts array elements with optional direction
    Sort(String, Option<bool>),
    /// Sorts ascending
    SortAsc(String),
    /// Sorts descending
    SortDesc(String),
    /// Swaps two items
    Swap(String, usize, usize),
    /// Transposes 2d array
    Transpose(String, String),
    /// Returns unique merged values
    Union(Vec<String>),
    /// Returns sliding windows
    Windows(String, usize),
}

impl Display for ArrayFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add(arr, val) => write!(f, "array::add({}, {})", arr, val),
            Self::All(arr, condition) => {
                if let Some(condition) = condition {
                    write!(f, "array::all({}, {})", arr, condition)
                }
                else {
                    write!(f, "array::all({})", arr)
                }
            }
            Self::Any(arr, condition) => {
                if let Some(condition) = condition {
                    write!(f, "array::any({}, {})", arr, condition)
                }
                else {
                    write!(f, "array::any({})", arr)
                }
            }
            Self::At(arr, idx) => write!(f, "array::at({}, {})", arr, idx),
            Self::Append(arr, val) => write!(f, "array::append({}, {})", arr, val),
            Self::BooleanAnd(arr1, arr2) => write!(f, "array::boolean_and({}, {})", arr1, arr2),
            Self::BooleanOr(arr1, arr2) => write!(f, "array::boolean_or({}, {})", arr1, arr2),
            Self::BooleanXor(arr1, arr2) => write!(f, "array::boolean_xor({}, {})", arr1, arr2),
            Self::BooleanNot(arr) => write!(f, "array::boolean_not({})", arr),
            Self::Combine(arrays) => write!(f, "array::combine([{}])", arrays.join(", ")),
            Self::Complement(arr1, arr2) => write!(f, "array::complement({}, {})", arr1, arr2),
            Self::Clump(arr, size) => write!(f, "array::clump({}, {})", arr, size),
            Self::Concat(arrays) => write!(f, "array::concat([{}])", arrays.join(", ")),
            Self::Difference(arr1, arr2) => write!(f, "array::difference({}, {})", arr1, arr2),
            Self::Distinct(arr) => write!(f, "array::distinct({})", arr),
            Self::Fill(arr, val, range) => {
                if let Some((start, end)) = range {
                    write!(f, "array::fill({}, {}, {}, {})", arr, val, start, end)
                }
                else {
                    write!(f, "array::fill({}, {})", arr, val)
                }
            }
            Self::Filter(arr, pattern) => write!(f, "array::filter({}, {})", arr, pattern),
            Self::FilterIndex(arr, pattern) => write!(f, "array::filter_index({}, {})", arr, pattern),
            Self::Find(arr, pattern) => write!(f, "array::find({}, {})", arr, pattern),
            Self::FindIndex(arr, pattern) => write!(f, "array::find_index({}, {})", arr, pattern),
            Self::First(arr) => write!(f, "array::first({})", arr),
            Self::Flatten(arr) => write!(f, "array::flatten({})", arr),
            Self::Fold(arr, initial, closure) => write!(f, "array::fold({}, {}, {})", arr, initial, closure),
            Self::Group(arr) => write!(f, "array::group({})", arr),
            Self::Insert(arr, idx, val) => write!(f, "array::insert({}, {}, {})", arr, idx, val),
            Self::Intersect(arrays) => write!(f, "array::intersect([{}])", arrays.join(", ")),
            Self::IsEmpty(arr) => write!(f, "array::is_empty({})", arr),
            Self::Join(arr, separator) => write!(f, "array::join({}, {})", arr, separator),
            Self::Last(arr) => write!(f, "array::last({})", arr),
            Self::Len(arr) => write!(f, "array::len({})", arr),
            Self::LogicalAnd(arr1, arr2) => write!(f, "array::logical_and({}, {})", arr1, arr2),
            Self::LogicalOr(arr1, arr2) => write!(f, "array::logical_or({}, {})", arr1, arr2),
            Self::LogicalXor(arr1, arr2) => write!(f, "array::logical_xor({}, {})", arr1, arr2),
            Self::Map(arr, closure) => write!(f, "array::map({}, {})", arr, closure),
            Self::Max(arr) => write!(f, "array::max({})", arr),
            Self::Matches(arr, pattern) => write!(f, "array::matches({}, {})", arr, pattern),
            Self::Min(arr) => write!(f, "array::min({})", arr),
            Self::Pop(arr) => write!(f, "array::pop({})", arr),
            Self::Prepend(arr, val) => write!(f, "array::prepend({}, {})", arr, val),
            Self::Push(arr, val) => write!(f, "array::push({}, {})", arr, val),
            Self::Range(start, end) => write!(f, "array::range({}, {})", start, end),
            Self::Reduce(arr, closure) => write!(f, "array::reduce({}, {})", arr, closure),
            Self::Remove(arr, idx) => write!(f, "array::remove({}, {})", arr, idx),
            Self::Repeat(arr, n) => write!(f, "array::repeat({}, {})", arr, n),
            Self::Reverse(arr) => write!(f, "array::reverse({})", arr),
            Self::Shuffle(arr) => write!(f, "array::shuffle({})", arr),
            Self::Slice(arr, start, end) => write!(f, "array::slice({}, {}, {})", arr, start, end),
            Self::Sort(arr, direction) => {
                if let Some(direction) = direction {
                    if *direction {
                        write!(f, "array::sort::asc({})", arr)
                    }
                    else {
                        write!(f, "array::sort::desc({})", arr)
                    }
                }
                else {
                    write!(f, "array::sort({})", arr)
                }
            }
            Self::SortAsc(arr) => write!(f, "array::sort::asc({})", arr),
            Self::SortDesc(arr) => write!(f, "array::sort::desc({})", arr),
            Self::Swap(arr, idx1, idx2) => write!(f, "array::swap({}, {}, {})", arr, idx1, idx2),
            Self::Transpose(arr1, arr2) => write!(f, "array::transpose({}, {})", arr1, arr2),
            Self::Union(arrays) => write!(f, "array::union([{}])", arrays.join(", ")),
            Self::Windows(arr, size) => write!(f, "array::windows({}, {})", arr, size),
        }
    }
}

impl Callable for ArrayFunction {
    fn namespace() -> &'static str { "array" }

    fn category(&self) -> &'static str {
        match self {
            // Basic array operations
            Self::Add(..) | Self::Append(..) | Self::Push(..) | Self::Prepend(..) => "mutation",
            Self::Pop(..) | Self::Remove(..) | Self::Insert(..) => "mutation",

            // Logical operations
            Self::All(..) | Self::Any(..) | Self::IsEmpty(..) => "logical",
            Self::LogicalAnd(..) | Self::LogicalOr(..) | Self::LogicalXor(..) => "logical",
            Self::BooleanAnd(..) | Self::BooleanOr(..) | Self::BooleanXor(..) | Self::BooleanNot(..) => "logical",

            // Transformation operations
            Self::Map(..) | Self::Filter(..) | Self::Reduce(..) | Self::Fold(..) => "transform",
            Self::Sort(..) | Self::SortAsc(..) | Self::SortDesc(..) => "transform",
            Self::Reverse(..) | Self::Shuffle(..) => "transform",
            Self::Repeat(..) => "transform",

            // Set operations
            Self::Union(..) | Self::Intersect(..) | Self::Difference(..) | Self::Complement(..) => "set",
            Self::Distinct(..) | Self::Group(..) => "set",

            // Analysis operations
            Self::Max(..) | Self::Min(..) | Self::Len(..) => "analysis",
            Self::First(..) | Self::Last(..) => "analysis",
            Self::Find(..) | Self::FindIndex(..) | Self::FilterIndex(..) => "analysis",

            // Window operations
            Self::Windows(..) | Self::Clump(..) | Self::Slice(..) => "window",

            // Combining operations
            Self::Combine(..) | Self::Concat(..) | Self::Join(..) => "combine",
            Self::Flatten(..) => "combine",

            // Range operations
            Self::Range(..) => "range",

            // Other operations
            Self::At(..) | Self::Matches(..) | Self::Fill(..) | Self::Transpose(..) | Self::Swap(..) => "utility",
        }
    }

    fn can_filter(&self) -> bool {
        matches!(
            self,
            // Logical operations that return boolean can be used in WHERE
            Self::All(..) |
            Self::Any(..) |
            Self::IsEmpty(..) |
            Self::LogicalAnd(..) |
            Self::LogicalOr(..) |
            Self::LogicalXor(..) |
            Self::BooleanAnd(..) |
            Self::BooleanOr(..) |
            Self::BooleanXor(..) |
            // Analysis operations that return boolean
            Self::Find(..) |
            Self::Matches(..)
        )
    }
}
