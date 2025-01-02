#![feature(const_type_id)]
pub use error::Error;
pub use error::Result;

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
        // Split into parts: DEFINE <TYPE> <NAME>
        let parts: Vec<&str> = stmt.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "DEFINE" {
            let def_type = parts[1]; // TABLE, FIELD, etc.
            let rest: Vec<&str> = parts[2..].to_vec();
            format!("DEFINE {} OVERWRITE {}", def_type, rest.join(" "))
        } else {
            stmt.to_string()
        }
    } else {
        stmt.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::MigrationManager;

    #[test]
    fn test_ensure_overwrite() {
        let cases = vec![
            (
                "DEFINE TABLE test TYPE NORMAL SCHEMAFULL",
                "DEFINE TABLE OVERWRITE test TYPE NORMAL SCHEMAFULL"
            ),
            (
                "DEFINE FIELD title ON TABLE test TYPE string",
                "DEFINE FIELD OVERWRITE title ON TABLE test TYPE string"
            ),
            (
                "DEFINE TABLE IF NOT EXISTS test TYPE NORMAL",
                "DEFINE TABLE OVERWRITE test TYPE NORMAL"
            ),
            (
                "DEFINE TABLE OVERWRITE test TYPE NORMAL",
                "DEFINE TABLE OVERWRITE test TYPE NORMAL"
            ),
            (
                "DEFINE FIELD OVERWRITE title ON TABLE test TYPE string",
                "DEFINE FIELD OVERWRITE title ON TABLE test TYPE string"
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(ensure_overwrite(input), expected, "Failed for input: {}", input);
        }
    }

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
