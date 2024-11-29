use crate::define::index::TableIndex;

#[derive(Default, Debug, Clone)]
pub struct IndexDefineStatement {
    pub(crate) table: Option<String>,
    pub(crate) index: TableIndex,
    pub(crate) index_type: Option<IndexType>,
    pub(crate) if_not_exists: bool,
}
pub trait IntoIndexColumn {
    fn into_index_column() -> IndexDefineStatement;
}
/// Specification of a Table index
#[derive(Debug, Clone)]
pub enum IndexType {
    Search,
    Mtree,
    HNSW,
}

impl IndexDefineStatement {
    /// Construct a new [`IndexDefineStatement`]
    pub fn new() -> Self {
        Self {
            table: None,
            index: Default::default(),
            index_type: None,
            if_not_exists: false,
        }
    }

    /// Define index if index not exists
    pub fn if_not_exists(&mut self) -> &mut Self {
        self.if_not_exists = true;
        self
    }

    /// Set index name
    pub fn name<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.index.name(name);
        self
    }

    pub fn table<T>(&mut self, table: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.table = Some(table.into());
        self
    }

    pub fn col<C>(&mut self, col: C) -> &mut Self
    where
        C: IntoIndexColumn,
    {
        self.index.col(col);
        self
    }
    pub fn full_text(&mut self) -> &mut Self {
        self.index_type(IndexType::Search)
    }

    pub fn index_type(&mut self, index_type: IndexType) -> &mut Self {
        self.index_type = Some(index_type);
        self
    }

    pub fn get_index_spec(&self) -> &TableIndex {
        &self.index
    }

    pub fn take(&mut self) -> Self {
        Self {
            table: self.table.take(),
            index: self.index.take(),
            index_type: self.index_type.take(),
            if_not_exists: self.if_not_exists,
        }
    }
}
