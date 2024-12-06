use anyhow::{anyhow, bail};
use tracing::{error, info};
use crate::SurrealDB;

#[derive(Debug, Clone, Default)]
pub struct DefineEventStatement {
    pub name: Option<String>,
    pub table: Option<String>,
    pub overwrite: bool,
    pub if_not_exists: bool,
    pub when: Option<String>,
    pub then: Option<String>,
    pub comment: Option<String>,
}

impl DefineEventStatement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
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

    pub fn when(mut self, when: impl Into<String>) -> Self {
        self.when = Some(when.into());
        self
    }

    pub fn then(mut self, then: impl Into<String>) -> Self {
        self.then = Some(then.into());
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE EVENT ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name.as_str());
        } else {
            bail!("Event name is required");
        }

        stmt.push_str(" ON TABLE ");
        if let Some(table) = &self.table {
            stmt.push_str(table.as_str());
        } else {
            bail!("Table name is required");
        }

        fn cleanup(s: impl Into<String>) -> String {
            let mut s = s.into().replace("\"","");
            s = s.replace("var:", "$");
            s
        }

        stmt.push_str(" WHEN ");
        stmt.push_str(&*cleanup(&self.when));

        stmt.push_str(" THEN ");
        stmt.push_str(&*cleanup(&self.then));

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
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