use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter, Write};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Any,
    Null,
    Bool,
    Bytes,
    Datetime,
    Decimal,
    Duration,
    Float,
    Int,
    Number,
    Object,
    Point,
    String,
    Uuid,
    Record(String),
    Geometry(String),
    Option(Box<FieldType>),
    Either(Vec<FieldType>),
    Set(Box<FieldType>, Option<u64>),
    Array(Box<FieldType>, Option<u64>),
    Function(Option<Vec<FieldType>>, Option<Box<FieldType>>),
    Range,
    Literal(Literal),
}

impl Default for FieldType {
    fn default() -> Self {
        Self::Any
    }
}

impl FieldType {
    /// Returns true if this type is an `any`
    pub(crate) fn is_any(&self) -> bool {
        matches!(self, FieldType::Any)
    }

    /// Returns true if this type is a record
    pub(crate) fn is_record(&self) -> bool {
        matches!(self, FieldType::Record(_))
    }

    /// Returns true if this type is optional
    pub(crate) fn can_be_none(&self) -> bool {
        matches!(self, FieldType::Option(_) | FieldType::Any)
    }

    /// Returns the kind in case of a literal, otherwise returns the kind itself
    fn to_kind(&self) -> Self {
        match self {
            FieldType::Literal(l) => l.to_kind(),
            k => k.to_owned(),
        }
    }

    /// Returns true if this type is a literal, or contains a literal
    pub(crate) fn is_literal_nested(&self) -> bool {
        if matches!(self, FieldType::Literal(_)) {
            return true;
        }

        if let FieldType::Option(x) = self {
            return x.is_literal_nested();
        }

        if let FieldType::Either(x) = self {
            return x.iter().any(|x| x.is_literal_nested());
        }

        false
    }

    /// Returns Some if this type can be converted into a discriminated object, None otherwise
    pub(crate) fn to_discriminated(&self) -> Option<FieldType> {
        match self {
            FieldType::Either(nested) => {
                if let Some(nested) = nested
                    .iter()
                    .map(|k| match k {
                        FieldType::Literal(Literal::Object(o)) => Some(o),
                        _ => None,
                    })
                    .collect::<Option<Vec<&BTreeMap<String, FieldType>>>>()
                {
                    if let Some(first) = nested.first() {
                        let mut key: Option<String> = None;

                        'key: for (k, v) in first.iter() {
                            let mut kinds: Vec<FieldType> = vec![v.to_owned()];
                            for item in nested[1..].iter() {
                                if let Some(kind) = item.get(k) {
                                    match kind {
                                        FieldType::Literal(l)
                                            if kinds.contains(&l.to_kind())
                                                || kinds.contains(&FieldType::Literal(
                                                    l.to_owned(),
                                                )) =>
                                        {
                                            continue 'key;
                                        }
                                        kind if kinds.iter().any(|k| *kind == k.to_kind()) => {
                                            continue 'key;
                                        }
                                        kind => {
                                            kinds.push(kind.to_owned());
                                        }
                                    }
                                } else {
                                    continue 'key;
                                }
                            }

                            key = Some(k.clone());
                            break;
                        }

                        if let Some(key) = key {
                            return Some(FieldType::Literal(Literal::DiscriminatedObject(
                                key.clone(),
                                nested.into_iter().map(|o| o.to_owned()).collect(),
                            )));
                        }
                    }
                }

                None
            }
            _ => None,
        }
    }

    // Return the kind of the contained value.
    //
    // For example: for `array<number>` or `set<number>` this returns `number`.
    // For `array<number> | set<float>` this returns `number | float`.
    pub(crate) fn inner_kind(&self) -> Option<FieldType> {
        let mut this = self;
        loop {
            match &this {
                FieldType::Any
                | FieldType::Null
                | FieldType::Bool
                | FieldType::Bytes
                | FieldType::Datetime
                | FieldType::Decimal
                | FieldType::Duration
                | FieldType::Float
                | FieldType::Int
                | FieldType::Number
                | FieldType::Object
                | FieldType::Point
                | FieldType::String
                | FieldType::Uuid
                | FieldType::Record(_)
                | FieldType::Geometry(_)
                | FieldType::Function(_, _)
                | FieldType::Range
                | FieldType::Literal(_) => return None,
                FieldType::Option(x) => {
                    this = x;
                }
                FieldType::Array(x, _) | FieldType::Set(x, _) => return Some(x.as_ref().clone()),
                FieldType::Either(x) => {
                    // a either shouldn't be able to contain a either itself so recursing here
                    // should be fine.
                    let kinds: Vec<FieldType> = x.iter().filter_map(Self::inner_kind).collect();
                    if kinds.is_empty() {
                        return None;
                    }
                    return Some(FieldType::Either(kinds));
                }
            }
        }
    }
}
impl From<&str> for FieldType {
    #[inline]
    fn from(v: &str) -> Self {
        match v {
            "any" => FieldType::Any,
            "null" => FieldType::Null,
            "bool" => FieldType::Bool,
            "bytes" => FieldType::Bytes,
            "datetime" => FieldType::Datetime,
            "decimal" => FieldType::Decimal,
            "duration" => FieldType::Duration,
            "float" => FieldType::Float,
            "int" => FieldType::Int,
            "number" => FieldType::Number,
            "object" => FieldType::Object,
            "point" => FieldType::Point,
            "string" => FieldType::String,
            "uuid" => FieldType::Uuid,
            "function" => FieldType::Function(None, None),
            s if s.starts_with("option<") => {
                let inner = s[7..s.len() - 1].to_string();
                FieldType::Option(Box::new(FieldType::from(inner)))
            }
            s if s.starts_with("record<") => {
                let inner = s[7..s.len() - 1].to_string();
                FieldType::Record(inner)
            }
            s if s.starts_with("geometry<") => {
                let inner = s[9..s.len() - 1].to_string();
                FieldType::Geometry(inner)
            }
            s if s.starts_with("set<") || s.starts_with("array<") => {
                let (kind, inner) = s.split_once('<').unwrap();
                let inner = inner[..inner.len() - 1].to_string();
                let parts: Vec<&str> = inner.split(',').collect();
                let inner_type = FieldType::from(parts[0].trim().to_string());
                let size = parts.get(1).and_then(|s| s.trim().parse().ok());
                match kind {
                    "set" => FieldType::Set(Box::new(inner_type), size),
                    "array" => FieldType::Array(Box::new(inner_type), size),
                    _ => unreachable!(),
                }
            }
            _ => {
                if v.contains('|') {
                    FieldType::Either(
                        v.split('|')
                            .map(|s| FieldType::from(s.trim().to_string()))
                            .collect(),
                    )
                } else {
                    // Default to string if no match
                    FieldType::String
                }
            }
        }
    }
}
impl From<&FieldType> for Box<FieldType> {
    #[inline]
    fn from(v: &FieldType) -> Self {
        Box::new(v.clone())
    }
}

impl From<String> for FieldType {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for FieldType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FieldType::Any => f.write_str("any"),
            FieldType::Null => f.write_str("null"),
            FieldType::Bool => f.write_str("bool"),
            FieldType::Bytes => f.write_str("bytes"),
            FieldType::Datetime => f.write_str("datetime"),
            FieldType::Decimal => f.write_str("decimal"),
            FieldType::Duration => f.write_str("duration"),
            FieldType::Float => f.write_str("float"),
            FieldType::Int => f.write_str("int"),
            FieldType::Number => f.write_str("number"),
            FieldType::Object => f.write_str("object"),
            FieldType::Point => f.write_str("point"),
            FieldType::String => f.write_str("string"),
            FieldType::Uuid => f.write_str("uuid"),
            FieldType::Function(_, _) => f.write_str("function"),
            FieldType::Option(k) => write!(f, "option<{}>", k),
            FieldType::Record(k) => match k {
                k if k.is_empty() => write!(f, "record"),
                k => write!(f, "record<{}>", k),
            },
            FieldType::Geometry(k) => match k {
                k if k.is_empty() => write!(f, "geometry"),
                k => write!(f, "geometry<{}>", k),
            },
            FieldType::Set(k, l) => match (k, l) {
                (k, None) if k.is_any() => write!(f, "set"),
                (k, None) => write!(f, "set<{k}>"),
                (k, Some(l)) => write!(f, "set<{k}, {l}>"),
            },
            FieldType::Array(k, l) => match (k, l) {
                (k, None) if k.is_any() => write!(f, "array"),
                (k, None) => write!(f, "array<{k}>"),
                (k, Some(l)) => write!(f, "array<{k}, {l}>"),
            },
            FieldType::Either(k) => write!(
                f,
                "{}",
                k.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(" | ")
            ),
            FieldType::Range => f.write_str("range"),
            FieldType::Literal(l) => write!(f, "{}", l),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Number(usize),
    Duration(Duration),
    Array(Vec<FieldType>),
    Object(BTreeMap<String, FieldType>),
    DiscriminatedObject(String, Vec<BTreeMap<String, FieldType>>),
}

impl Literal {
    pub fn to_kind(&self) -> FieldType {
        match self {
            Self::String(_) => FieldType::String,
            Self::Number(_) => FieldType::Number,
            Self::Duration(_) => FieldType::Duration,
            Self::Array(a) => {
                if let Some(inner) = a.first() {
                    if a.iter().all(|x| x == inner) {
                        return FieldType::Array(Box::new(inner.to_owned()), Some(a.len() as u64));
                    }
                }

                FieldType::Array(Box::new(FieldType::Any), None)
            }
            Self::Object(_) => FieldType::Object,
            Self::DiscriminatedObject(_, _) => FieldType::Object,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Literal::String(s) => write!(f, "{}", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Duration(n) => write!(f, "{}", n.as_secs()),
            Literal::Array(a) => {
                f.write_char('[')?;
                if !a.is_empty() {
                    write!(
                        f,
                        "{}",
                        a.iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                    )?;
                }
                f.write_char(']')
            }
            Literal::Object(o) => {
                f.write_char('{')?;
                if !o.is_empty() {
                    write!(
                        f,
                        "{}",
                        o.iter()
                            .map(|(k, v)| format!("{}: {}", k, v))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )?;
                }

                f.write_char('}')
            }
            Literal::DiscriminatedObject(_, discriminants) => {
                for (i, o) in discriminants.iter().enumerate() {
                    if i > 0 {
                        f.write_str(" | ")?;
                    }

                    f.write_char('{')?;
                    if !o.is_empty() {
                        write!(
                            f,
                            "{}",
                            o.iter()
                                .map(|(k, v)| format!("{}: {}", k, v,))
                                .collect::<Vec<String>>()
                                .join(", ")
                        )?;
                    }

                    f.write_char('}')?;
                }

                Ok(())
            }
        }
    }
}
