#![feature(const_type_id)]
pub use error::Error;
pub use error::Result;
use magritte::{
    HasEvents, HasIndexes
    ,
};
use std::any::Any;

pub mod edge;
pub mod error;
pub mod introspection;
pub mod manager;
pub mod snapshot;
pub mod table;
pub mod test_models;
pub mod types;

pub(crate) fn ensure_overwrite(stmt: &str) -> String {
    let stmt = stmt.trim();
    if stmt.contains("IF NOT EXISTS") {
        stmt.replace("IF NOT EXISTS", "OVERWRITE")
    } else if !stmt.contains("OVERWRITE") {
        // Insert OVERWRITE after the first DEFINE keyword
        let mut parts = stmt.splitn(2, "DEFINE");
        let after_define = parts.nth(1).unwrap_or("");
        format!("DEFINE OVERWRITE{}", after_define)
    } else {
        stmt.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::MigrationManager;

    #[test]
    fn test_get_file_stem() {
        let pair = vec![
            (
                "m20220101_000001_create_table.rs",
                "m20220101_000001_create_table",
            ),
            (
                "src/m20220101_000001_create_table.rs",
                "m20220101_000001_create_table",
            ),
            (
                "migration/src/m20220101_000001_create_table.rs",
                "m20220101_000001_create_table",
            ),
            (
                "/migration/src/m20220101_000001_create_table.tmp.rs",
                "m20220101_000001_create_table.tmp",
            ),
        ];
        for (path, expect) in pair {
            assert_eq!(MigrationManager::get_file_stem(path), expect);
        }
    }
}
