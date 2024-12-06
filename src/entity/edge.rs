use crate::entity::HasColumns;
use crate::prelude::ColumnTrait;
use magritte_query::define::define_table::DefineTableStatement;
use magritte_query::types::{EdgeType, Permission, SchemaType, TableType};
use std::fmt::{Debug, Display};
use std::time::Duration;
use magritte_query::define_edge::DefineEdgeStatement;

/// Defines an Edge for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeDef {
    pub(crate) name: String,
    pub(crate) schema_type: SchemaType,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) as_select: Option<String>,
    pub(crate) changefeed: Option<(Duration, bool)>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) drop: bool,
    pub(crate) enforced: bool,
    pub(crate) comment: Option<String>,
}

pub trait EdgeTrait: EdgeType + HasColumns {
    type EntityFrom: TableType;
    type EntityTo: TableType;
    fn def() -> EdgeDef;
    fn def_owned(&self) -> EdgeDef {
        Self::def()
    }
    fn to_statement() -> DefineEdgeStatement<Self> {
        Self::def().to_statement()
    }

    fn to_statement_owned(&self) -> DefineEdgeStatement<Self> {
        self.def_owned().to_statement()
    }

    fn columns(&self) -> impl IntoIterator<Item = impl ColumnTrait>
    where
        Self: Sized,
    {
        <Self as HasColumns>::columns()
    }
}

impl EdgeDef {
    pub fn new(
        name: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        schema_type: impl Into<String>,
        permissions: Option<Vec<String>>,
        overwrite: bool,
        if_not_exists: bool,
        drop: bool,
        enforced: bool,
        as_select: Option<String>,
        changefeed: Option<(u64, bool)>,
        comment: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            schema_type: SchemaType::from(schema_type.into()),
            permissions: permissions
                .as_ref()
                .map(|pers| pers.iter().map(|c| Permission::from(c)).collect()),
            from: from.into(),
            to: to.into(),
            as_select,
            changefeed: changefeed.map(|c| (Duration::from_mins(c.0), c.1)),
            overwrite,
            if_not_exists,
            drop,
            enforced,
            comment,
        }
    }
    pub fn edge_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn schema_type(&self) -> &SchemaType {
        &self.schema_type
    }
    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_deref().unwrap_or(&[])
    }
    pub fn edge_from(&self) -> &str {
        self.from.as_str()
    }
    pub fn edge_to(&self) -> &str {
        self.to.as_str()
    }
    pub fn is_enforced(&self) -> bool {
        self.enforced
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }
    pub fn is_drop(&self) -> bool {
        self.drop
    }
    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
    pub fn as_select(&self) -> Option<&str> {
        self.as_select.as_ref().map(|s| s.as_str())
    }

    pub fn changefeed(&self) -> Option<(Duration, bool)> {
        self.changefeed
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|s| s.as_str())
    }
    pub fn to_statement<T: EdgeTrait>(&self) -> DefineEdgeStatement<T> {
        let mut def = DefineEdgeStatement::new();

        if self.overwrite {
            def = def.overwrite()
        } else if self.if_not_exists {
            def = def.if_not_exists()
        } else if self.drop {
            def = def.drop()
        }

        def = def.name(self.name.clone());
        def = def.schema_type(self.schema_type.clone());

        def = def.from(self.from.clone());
        def = def.to(self.to.clone());

        if let Some(as_select) = &self.as_select {
            def = def.as_select(as_select.as_str().parse().unwrap());
        }
        if let Some(changefeed) = &self.changefeed {
            def = def.changefeed(changefeed.0, changefeed.1);
        }

        if let Some(permissions) = &self.permissions {
            def = def.permissions(permissions.clone());
        }

        if let Some(comment) = &self.comment {
            def = def.comment(comment.clone());
        }

        if self.enforced {
            def = def.enforced();
        }
        def
    }
}
