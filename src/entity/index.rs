use anyhow::bail;
use anyhow::Result;
use magritte_query::types::{IndexType, TableType};
use magritte_query::vector_search::{VectorDistance, VectorType};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

/// Defines an Index for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct IndexDef {
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) overwrite: bool,
    pub(crate) use_table: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) fields: Option<Vec<String>>,
    pub(crate) columns: Option<Vec<String>>,
    pub(crate) unique: bool,
    pub(crate) specifics: IndexSpecifics,
    pub(crate) comment: Option<String>,
    pub(crate) concurrently: bool,
}

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
pub trait IndexTrait: IndexType {
    type EntityName: TableType;

    fn def(&self) -> IndexDef;

    fn to_statement(&self) -> Result<String> {
        self.def().to_statement()
    }
}

impl IndexDef {
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        fields: Option<Vec<String>>,
        columns: Option<Vec<String>>,
        overwrite: bool,
        use_table: bool,
        if_not_exists: bool,
        unique: bool,
        specifics: String,
        comment: Option<String>,
        concurrently: bool,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            overwrite,
            use_table,
            if_not_exists,
            fields,
            columns,
            unique,
            specifics: IndexSpecifics::from(&specifics),
            comment,
            concurrently,
        }
    }
    pub fn index_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn table_name(&self) -> &str {
        self.table.as_str()
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
    pub fn is_concurrent(&self) -> bool {
        self.concurrently
    }
    pub fn fields(&self) -> Option<Vec<&str>> {
        self.fields.as_ref().map(|fields| fields.iter().map(|f| f.as_str()).collect())
    }
    pub fn columns(&self) -> Option<Vec<&str>> {
        self.columns.as_ref().map(|columns| columns.iter().map(|c| c.as_str()).collect())
    }
    pub fn specifics(&self) -> &IndexSpecifics {
        &self.specifics
    }
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|c| c.as_str())
    }
    pub fn to_statement(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE INDEX ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        stmt.push_str(&*self.name);

        stmt.push_str(" ON ");
        if self.use_table {
            stmt.push_str("TABLE ");
        }
        stmt.push_str(&*self.table);

        if let Some(fields) = &self.fields {
            stmt.push_str(" FIELDS ");
            if fields.len() == 1 {
                stmt.push_str(fields.first().unwrap().as_str());
            } else if fields.len() > 1 {
                stmt.push_str(fields.join(", ").as_str());
            }
        } else if let Some(columns) = &self.columns {
            stmt.push_str(" COLUMNS ");
            if columns.len() == 1 {
                stmt.push_str(columns.first().unwrap().as_str());
            } else if columns.len() > 1 {
                stmt.push_str(columns.join(", ").as_str());
            }
        } else {
            bail!("No fields or columns provided")
        }

        stmt.push_str(self.specifics.to_string().as_str());

        if self.unique {
            stmt.push_str(" UNIQUE");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        if self.concurrently {
            stmt.push_str(" CONCURRENTLY");
        }

        stmt.push(';');
        Ok(stmt)
    }
}
