use std::fmt::Display;
use anyhow::{anyhow, bail};
use tracing::{error, info};
use crate::{EdgeType, IndexSpecifics, SurrealDB};
use crate::define_edge::DefineEdgeStatement;

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
        Self { table: None, name: None, if_not_exists: false, fields: None, columns: None, unique: false, specifics: Default::default(), comment: None, overwrite: false, concurrently: false }
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
        stmt.push_str("DEFINE INDEX ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        stmt.push_str(&*self.name);

        stmt.push_str(" ON ");

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


    pub async fn execute(self, conn: SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        let query = self.build()?;
        info!("Executing query: {}", query);

        let mut surreal_query = conn.query(query);

        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Query execution failed: {:?}", e);
                Err(anyhow!(e))
            }
        }
    }
}


impl Display for DefineIndexStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}
