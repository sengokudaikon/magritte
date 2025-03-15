use magritte_db::{db, QueryType, SurrealDB};
use crate::IndexSpecifics;
use anyhow::bail;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
pub struct DefineIndexStatement {
    pub(crate) name: Option<String>,
    pub(crate) table: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) fields: Option<Vec<String>>,
    pub(crate) columns: Option<Vec<String>>,
    pub(crate) unique: bool,
    pub(crate) specifics: IndexSpecifics,
    pub(crate) comment: Option<String>,
    pub(crate) concurrently: bool,
}

impl DefineIndexStatement {
    pub fn new() -> Self {
        Self {
            table: None,
            name: None,
            if_not_exists: false,
            fields: None,
            columns: None,
            unique: false,
            specifics: Default::default(),
            comment: None,
            overwrite: false,
            concurrently: false,
        }
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn fields(mut self, fields: impl Into<Vec<String>>) -> Self {
        self.fields = Some(fields.into());
        self
    }

    pub fn columns(mut self, columns: impl Into<Vec<String>>) -> Self {
        self.columns = Some(columns.into());
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn specifics(mut self, specifics: impl Into<IndexSpecifics>) -> Self {
        self.specifics = specifics.into();
        self
    }

    pub fn concurrently(mut self) -> Self {
        self.concurrently = true;
        self
    }

    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        if self.name.is_none() {
            return Ok(stmt);
        }
        stmt.push_str("DEFINE INDEX ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        if let Some(name) = &self.name {
            stmt.push_str(name.as_str());
        } else {
            bail!("Index name is required");
        }

        stmt.push_str(" ON ");

        if let Some(table) = &self.table {
            stmt.push_str(table.as_str());
        } else {
            bail!("Table name is required");
        }

        if let Some(fields) = &self.fields {
            stmt.push_str(" FIELDS ");
            match fields.len() {
                1 => stmt.push_str(fields.first().unwrap().as_str()),
                n if n > 1 => stmt.push_str(fields.join(", ").as_str()),
                _ => {}
            }
        } else if let Some(columns) = &self.columns {
            stmt.push_str(" COLUMNS ");
            match columns.len() {
                1 => stmt.push_str(columns.first().unwrap().as_str()),
                n if n > 1 => stmt.push_str(columns.join(", ").as_str()),
                _ => {}
            }
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

    pub async fn execute(self, ) -> anyhow::Result<Vec<serde_json::Value>> {
        db().execute(self.build()?, vec![]).await
    }
}

impl Display for DefineIndexStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::types::index::IndexSpecifics;
    use crate::backend::vector_search::{VectorDistance, VectorType};

    #[test]
    fn test_empty_index() {
        let stmt = DefineIndexStatement::new();
        assert_eq!(stmt.build().unwrap(), "");
    }

    #[test]
    fn test_basic_index() {
        let stmt = DefineIndexStatement::new()
            .name("userAgeIndex")
            .table("user")
            .columns(vec!["age".to_string()]);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userAgeIndex ON user COLUMNS age;");
    }

    #[test]
    fn test_fields_alternative() {
        let stmt = DefineIndexStatement::new()
            .name("userAgeIndex")
            .table("user")
            .fields(vec!["age".to_string()]);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userAgeIndex ON user FIELDS age;");
    }

    #[test]
    fn test_unique_index() {
        let stmt = DefineIndexStatement::new()
            .name("userEmailIndex")
            .table("user")
            .columns(vec!["email".to_string()])
            .unique();
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userEmailIndex ON user COLUMNS email UNIQUE;");
    }

    #[test]
    fn test_composite_index() {
        let stmt = DefineIndexStatement::new()
            .name("test")
            .table("user")
            .fields(vec!["account".to_string(), "email".to_string()]);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX test ON user FIELDS account, email;");
    }

    #[test]
    fn test_composite_unique_index() {
        let stmt = DefineIndexStatement::new()
            .name("test")
            .table("user")
            .fields(vec!["account".to_string(), "email".to_string()])
            .unique();
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX test ON user FIELDS account, email UNIQUE;");
    }

    #[test]
    fn test_index_with_comment() {
        let stmt = DefineIndexStatement::new()
            .name("userEmailIndex")
            .table("user")
            .columns(vec!["email".to_string()])
            .unique()
            .comment("Ensures email uniqueness");
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userEmailIndex ON user COLUMNS email UNIQUE COMMENT \"Ensures email uniqueness\";");
    }

    #[test]
    fn test_index_with_overwrite() {
        let stmt = DefineIndexStatement::new()
            .name("userEmailIndex")
            .table("user")
            .columns(vec!["email".to_string()])
            .overwrite();
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX OVERWRITE userEmailIndex ON user COLUMNS email;");
    }

    #[test]
    fn test_index_if_not_exists() {
        let stmt = DefineIndexStatement::new()
            .name("userEmailIndex")
            .table("user")
            .columns(vec!["email".to_string()])
            .if_not_exists();
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX IF NOT EXISTS userEmailIndex ON user COLUMNS email;");
    }

    #[test]
    fn test_index_concurrently() {
        let stmt = DefineIndexStatement::new()
            .name("userEmailIndex")
            .table("user")
            .columns(vec!["email".to_string()])
            .concurrently();
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userEmailIndex ON user COLUMNS email CONCURRENTLY;");
    }

    #[test]
    fn test_search_index() {
        let specifics = IndexSpecifics::Search {
            analyzer: Some("ascii".to_string()),
            bm25: None,
            highlights: true,
        };
        
        let stmt = DefineIndexStatement::new()
            .name("userNameIndex")
            .table("user")
            .columns(vec!["name".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userNameIndex ON user COLUMNS name SEARCH ANALYZER ascii  HIGHLIGHTS;");
    }

    #[test]
    fn test_search_index_with_bm25() {
        let specifics = IndexSpecifics::Search {
            analyzer: Some("ascii".to_string()),
            bm25: Some((1.2, 0.75)), // common BM25 parameters
            highlights: true,
        };
        
        let stmt = DefineIndexStatement::new()
            .name("userNameIndex")
            .table("user")
            .columns(vec!["name".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX userNameIndex ON user COLUMNS name SEARCH ANALYZER ascii BM25(1.2, 0.75) HIGHLIGHTS;");
    }

    #[test]
    fn test_mtree_index() {
        let specifics = IndexSpecifics::MTREE {
            dimension: 3,
            vector_type: None,
            dist: VectorDistance::Euclidean,
            capacity: None,
        };
        
        let stmt = DefineIndexStatement::new()
            .name("mt_pt")
            .table("pts")
            .fields(vec!["point".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX mt_pt ON pts FIELDS point MTREE DIMENSION 3DIST euclidean;");
    }

    #[test]
    fn test_mtree_index_with_distance() {
        let specifics = IndexSpecifics::MTREE {
            dimension: 4,
            vector_type: None,
            dist: VectorDistance::Manhattan,
            capacity: None,
        };
        
        let stmt = DefineIndexStatement::new()
            .name("idx_mtree_embedding_manhattan")
            .table("Document")
            .fields(vec!["items.embedding".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX idx_mtree_embedding_manhattan ON Document FIELDS items.embedding MTREE DIMENSION 4DIST manhattan;");
    }

    #[test]
    fn test_mtree_index_with_type_and_capacity() {
        let specifics = IndexSpecifics::MTREE {
            dimension: 4,
            vector_type: Some(VectorType::I64),
            dist: VectorDistance::Euclidean,
            capacity: Some(50),
        };
        
        let stmt = DefineIndexStatement::new()
            .name("idx_mtree_embedding")
            .table("Document")
            .fields(vec!["items.embedding".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX idx_mtree_embedding ON Document FIELDS items.embedding MTREE DIMENSION 4 TYPE i64DIST euclidean CAPACITY 50;");
    }

    #[test]
    fn test_hnsw_index() {
        let specifics = IndexSpecifics::HNSW {
            dimension: 4,
            vector_type: None,
            dist: VectorDistance::Euclidean,
            efc: None,
            m: None,
        };
        
        let stmt = DefineIndexStatement::new()
            .name("mt_pts")
            .table("pts")
            .fields(vec!["point".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX mt_pts ON pts FIELDS point HNSW DIMENSION 4DIST euclidean;");
    }

    #[test]
    fn test_hnsw_index_with_parameters() {
        let specifics = IndexSpecifics::HNSW {
            dimension: 4,
            vector_type: None,
            dist: VectorDistance::Euclidean,
            efc: Some(150),
            m: Some(12),
        };
        
        let stmt = DefineIndexStatement::new()
            .name("mt_pts")
            .table("pts")
            .fields(vec!["point".to_string()])
            .specifics(specifics);
        
        assert_eq!(stmt.to_string(), "DEFINE INDEX mt_pts ON pts FIELDS point HNSW DIMENSION 4DIST euclidean EFC 150 M 12;");
    }

    #[test]
    fn test_missing_table() {
        let stmt = DefineIndexStatement::new()
            .name("userEmailIndex");
            
        assert!(stmt.build().unwrap_err().to_string().contains("Table name is required"));
    }

    #[test]
    fn test_complex_index() {
        let specifics = IndexSpecifics::MTREE {
            dimension: 3,
            vector_type: Some(VectorType::F32),
            dist: VectorDistance::Cosine,
            capacity: Some(100),
        };
        
        let stmt = DefineIndexStatement::new()
            .name("complex_index")
            .table("vectors")
            .fields(vec!["embedding".to_string()])
            .specifics(specifics)
            .comment("Vector similarity search index")
            .concurrently();
        
        assert_eq!(
            stmt.to_string(), 
            "DEFINE INDEX complex_index ON vectors FIELDS embedding MTREE DIMENSION 3 TYPE f32DIST cosine CAPACITY 100 COMMENT \"Vector similarity search index\" CONCURRENTLY;"
        );
    }
}
