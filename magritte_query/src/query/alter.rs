use anyhow::Result;
use magritte_core::transaction::Transactional;
use magritte_core::{Permission, SchemaType};
use magritte_db::db;
use serde::de::DeserializeOwned;
use tracing::instrument;

/// ALTER query builder with allowed method chains
#[derive(Clone, Debug)]
pub struct AlterStatement {
    table: Option<String>,
    if_exists: bool,
    drop: bool,
    schema_type: Option<SchemaType>,
    permissions: Vec<Permission>,
    comment: Option<String>,
    in_transaction: bool,
}

impl AlterStatement {
    /// Specify Table to alter
    #[instrument(skip(self))]
    pub fn table(mut self, name: &str) -> Self {
        self.table = Some(name.to_string());
        self
    }

    /// Add IF EXISTS clause
    #[instrument(skip(self))]
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    /// Mark Table as DROP
    #[instrument(skip(self))]
    pub fn drop(mut self) -> Self {
        self.drop = true;
        self
    }

    /// Set Table as SCHEMAFULL
    #[instrument(skip(self))]
    pub fn schemafull(mut self) -> Self {
        self.schema_type = Some(SchemaType::Schemafull);
        self
    }

    /// Set Table as SCHEMALESS
    #[instrument(skip(self))]
    pub fn schemaless(mut self) -> Self {
        self.schema_type = Some(SchemaType::Schemaless);
        self
    }

    /// Add PERMISSIONS clause
    #[instrument(skip(self))]
    pub fn permissions(mut self, perms: Vec<Permission>) -> Self {
        self.permissions = perms;
        self
    }

    /// Add COMMENT
    #[instrument(skip(self))]
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }
}

impl AlterStatement {
    #[instrument(skip_all)]
    pub fn new() -> Self {
        Self {
            table: None,
            drop: false,
            if_exists: false,
            schema_type: None,
            permissions: Vec::new(),
            comment: None,
            in_transaction: false,
        }
    }

    /// Build the ALTER query

    #[instrument(skip_all)]
    pub fn build(&self) -> Result<String> {
        let mut query = String::from("ALTER TABLE");

        // Add IF EXISTS
        if self.if_exists {
            query.push_str(" IF EXISTS")
        }

        // Add Table name
        if let Some(table) = &self.table {
            query.push_str(&format!(" {}", table));
        }

        // Add DROP if specified
        if self.drop {
            query.push_str(" DROP");
        }

        // Add schema type
        if let Some(schema_type) = &self.schema_type {
            match schema_type {
                SchemaType::Schemafull => query.push_str(" SCHEMAFULL"),
                SchemaType::Schemaless => query.push_str(" SCHEMALESS"),
            }
        }

        // Add permissions
        if !self.permissions.is_empty() {
            query.push_str(" PERMISSIONS");
            for perm in &self.permissions {
                match perm {
                    Permission::None => query.push_str(" NONE"),
                    Permission::Full => query.push_str(" FULL"),
                    Permission::Select(expr) => query.push_str(&format!(" FOR select {}", expr)),
                    Permission::Create(expr) => query.push_str(&format!(" FOR create {}", expr)),
                    Permission::Update(expr) => query.push_str(&format!(" FOR update {}", expr)),
                    Permission::Delete(expr) => query.push_str(&format!(" FOR delete {}", expr)),
                }
            }
        }

        // Add comment if specified
        if let Some(comment) = &self.comment {
            query.push_str(&format!(" COMMENT '{}'", comment));
        }

        Ok(query)
    }

    /// Execute the ALTER query
    #[instrument(skip_all)]
    async fn execute<T: Send + DeserializeOwned + 'static>(self) -> Result<Vec<T>> {
        db().execute(self.build()?, vec![]).await
    }
}
impl Transactional for AlterStatement {
    fn is_transaction(&self) -> bool {
        self.in_transaction
    }

    fn in_transaction(&mut self) -> &mut bool {
        &mut self.in_transaction
    }
}
