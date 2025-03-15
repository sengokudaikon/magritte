use crate::define_table::AsSelect;
use anyhow::bail;
use magritte_core::{EdgeType, Permission, SchemaType};
use magritte_db::db;
use std::fmt::Display;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DefineEdgeStatement<T: EdgeType> {
    pub(crate) name: Option<String>,
    pub(crate) schema_type: Option<SchemaType>,
    pub(crate) from: Option<String>,
    pub(crate) to: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) as_select: Option<AsSelect>,
    pub(crate) changefeed: Option<(Duration, bool)>,
    pub(crate) drop: bool,
    pub(crate) enforced: bool,
    pub(crate) comment: Option<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for DefineEdgeStatement<T>
where
    T: EdgeType,
{
    fn default() -> Self {
        Self {
            name: None,
            schema_type: None,
            from: None,
            to: None,
            overwrite: false,
            if_not_exists: false,
            permissions: None,
            as_select: None,
            changefeed: None,
            drop: false,
            enforced: false,
            comment: None,
            _marker: Default::default(),
        }
    }
}

impl<T> DefineEdgeStatement<T>
where
    T: EdgeType,
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

    pub fn get_from(&self) -> Option<String> {
        self.from.clone()
    }

    pub fn get_to(&self) -> Option<String> {
        self.to.clone()
    }

    pub fn get_enforced(&self) -> bool {
        self.enforced
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

impl<T> DefineEdgeStatement<T>
where
    T: EdgeType,
{
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn schema_type(mut self, schema_type: SchemaType) -> Self {
        self.schema_type = Some(schema_type);
        self
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
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

    pub fn enforced(mut self) -> Self {
        self.enforced = true;
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

        stmt.push_str(" TYPE RELATION");

        if let Some(schema_type) = &self.schema_type {
            stmt.push(' ');
            stmt.push_str(schema_type.to_string().as_str());
        } else {
            stmt.push_str(" SCHEMALESS ")
        }

        if let Some(from) = &self.from {
            stmt.push_str(format!(" FROM {}", from).as_str());
        } else {
            bail!("From is required");
        }
        if let Some(to) = &self.to {
            stmt.push_str(format!(" TO {}", to).as_str());
        } else {
            bail!("To is required");
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

        if self.enforced {
            stmt.push_str(" ENFORCED");
        }

        stmt.push(';');
        Ok(stmt)
    }

    pub async fn execute(self) -> anyhow::Result<Vec<T>> {
        db().execute(self.build()?, vec![]).await
    }
}

impl<T> Display for DefineEdgeStatement<T>
where
    T: EdgeType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}
