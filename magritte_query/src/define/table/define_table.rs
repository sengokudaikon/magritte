use crate::{Permission, SchemaType, TableType};
use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info};
use crate::database::{QueryType, SurrealDB};

#[derive(Debug, Default, Serialize, Deserialize, Hash, Clone, Eq, PartialEq, PartialOrd)]
pub struct AsSelect {
    pub projections: String,
    pub from: String,
    pub where_: Option<String>,
    pub group_by: Option<String>,
}
impl Display for AsSelect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut query = String::new();
        query.push_str(&self.projections.to_string());
        query.push_str(&format!(" FROM {}", &self.from));
        if let Some(where_clause) = &self.where_ {
            query.push_str(&format!(" WHERE {}", where_clause));
        }
        if let Some(group_by) = &self.group_by {
            query.push_str(&format!(" GROUP BY {}", group_by));
        }
        write!(f, "{}", query)
    }
}

impl FromStr for AsSelect {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut projections = String::new();
        let mut from = String::new();
        let mut where_ = None;
        let mut group_by = None;
        for (i, line) in s.lines().enumerate() {
            if i == 0 {
                projections = line.to_string();
            } else if i == 1 {
                from = line.to_string();
            } else if line.starts_with("WHERE") {
                where_ = Some(line.to_string());
            } else if line.starts_with("GROUP BY") {
                group_by = Some(line.to_string());
            }
        }
        Ok(AsSelect {
            projections,
            from,
            where_,
            group_by,
        })
    }
}
#[derive(Clone, Debug)]
pub struct DefineTableStatement<T: TableType> {
    pub(crate) name: Option<String>,
    pub(crate) schema_type: Option<SchemaType>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) drop: bool,
    pub(crate) as_select: Option<AsSelect>,
    pub(crate) changefeed: Option<(Duration, bool)>,
    pub(crate) comment: Option<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for DefineTableStatement<T>
where
    T: TableType,
{
    fn default() -> Self {
        Self {
            name: None,
            schema_type: None,
            overwrite: false,
            if_not_exists: false,
            permissions: None,
            drop: false,
            as_select: None,
            changefeed: None,
            comment: None,
            _marker: Default::default(),
        }
    }
}

impl<T> DefineTableStatement<T>
where
    T: TableType,
{
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn get_schema_type(&self) -> Option<SchemaType> {
        self.schema_type.clone()
    }
    pub fn get_overwrite(&self) -> bool {
        self.overwrite
    }

    pub fn get_if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn get_permissions(&self) -> Option<Vec<Permission>> {
        self.permissions.clone()
    }

    pub fn get_drop(&self) -> bool {
        self.drop
    }

    pub fn get_as_select(&self) -> Option<AsSelect> {
        self.as_select.clone()
    }

    pub fn get_changefeed(&self) -> Option<(Duration, bool)> {
        self.changefeed
    }

    pub fn get_comment(&self) -> Option<String> {
        self.comment.clone()
    }
}

impl<T> DefineTableStatement<T>
where
    T: TableType,
{
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn schema_type(mut self, schema_type: SchemaType) -> Self {
        self.schema_type = Some(schema_type);
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

    pub fn permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn drop(mut self) -> Self {
        self.drop = true;
        self
    }

    pub fn as_select(mut self, as_select: AsSelect) -> Self {
        self.as_select = Some(as_select);
        self
    }

    pub fn changefeed(mut self, duration: Duration, include_original: bool) -> Self {
        self.changefeed = Some((duration, include_original));
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE TABLE ");

        if self.drop {
            stmt.push_str("DROP ");
        }
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }

        if let Some(name) = &self.name {
            stmt.push_str(name.as_str());
        } else {
            bail!("Table name is required");
        }

        stmt.push_str(" TYPE NORMAL");

        if let Some(schema_type) = &self.schema_type {
            stmt.push(' ');
            stmt.push_str(schema_type.to_string().as_str());
        } else {
            stmt.push_str(" SCHEMALESS ")
        }

        if let Some(as_select) = &self.as_select {
            stmt.push_str(format!(" AS SELECT {}", as_select).as_str());
        }
        if let Some(changefeed) = &self.changefeed {
            stmt.push_str(format!("CHANGEFEED {}", changefeed.0.as_secs()).as_str());
            if changefeed.1 {
                stmt.push_str(" INCLUDE ORIGINAL ");
            }
        }

        if let Some(permissions) = &self.permissions {
            if !permissions.is_empty() {
                stmt.push_str(" PERMISSIONS ");
                for perms in permissions {
                    stmt.push_str(format!("{}", perms).as_str());
                }
            }
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }
        stmt.push(';');
        Ok(stmt)
    }

    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<T>> {
        conn.execute(self.build()?, vec![], QueryType::Schema, None).await
    }
}

impl<T> Display for DefineTableStatement<T>
where
    T: TableType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}
