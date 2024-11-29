//DEFINE INDEX [ OVERWRITE | IF NOT EXISTS ] @name ON [ TABLE ] @Table [ FIELDS | COLUMNS ]
// @fields
// 	[ UNIQUE
//         | SEARCH ANALYZER @analyzer [ BM25 [(@k1, @b)] ] [ HIGHLIGHTS ]
//         | MTREE DIMENSION @dimension [ TYPE @type ] [ DIST @distance ] [ CAPACITY @capacity]
//         | HNSW DIMENSION @dimension [ TYPE @type ] [DIST @distance] [ EFC @efc ] [ M @m ]
//     ]
//     [ COMMENT @string ]
//     [ CONCURRENTLY ]
#[derive(Clone, Debug, Default)]
pub struct TableIndex {
    name: String,
}

impl TableIndex {
    pub(crate) fn col<T>(&self, _p0: T) {
        todo!()
    }
    pub(crate) fn name<T>(&self, _p0: T)
    where
        T: Into<String>,
    {
        todo!()
    }
    pub(crate) fn take(&mut self) -> Self {
        todo!()
    }
}
