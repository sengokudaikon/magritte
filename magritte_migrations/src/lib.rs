use crate::edge::EdgeDiff;
use crate::table::TableDiff;
pub use error::Error;
pub use error::Result;
use magritte::{EdgeRegistration, SchemaSnapshot, TableRegistration};

pub mod edge;
pub mod error;
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

pub fn current_schema_from_code() -> Result<SchemaSnapshot> {
    let mut schema = SchemaSnapshot::new();
    for reg in inventory::iter::<TableRegistration> {
        let table_snap = (reg.builder)().map_err(Error::from)?;
        schema.add_table(table_snap);
    }
    for reg in inventory::iter::<EdgeRegistration> {
        let edge_snap = (reg.builder)().map_err(Error::from)?;
        schema.add_edge(edge_snap);
    }
    Ok(schema)
}
pub fn get_file_stem(path: &str) -> &str {
    std::path::Path::new(path)
        .file_stem()
        .map(|f| f.to_str().unwrap())
        .unwrap()
}

pub fn generate_diff_migration(
    old_snapshot: &SchemaSnapshot,
    new_snapshot: &SchemaSnapshot,
) -> Result<Vec<String>> {
    let mut statements = Vec::new();

    // Compare tables
    for (table_name, new_table) in &new_snapshot.tables {
        if let Some(old_table) = old_snapshot.tables.get(table_name) {
            // Build TableDiff
            let diff = TableDiff::from_snapshots(old_table, new_table)?;
            let up_stmts = diff.generate_statements(table_name)?;
            statements.extend(up_stmts);
        } else {
            // Table didn't exist before
            let mut diff = TableDiff::new(None, Some(new_table.define_table_statement.clone()));
            // Populate diff fields etc. if needed, or if fields are already in the `from_snapshots` logic, no need.
            // Actually, since we don't have an old snapshot, we just treat all fields/indexes/events as "added".
            for (f, v) in &new_table.fields {
                diff.added_columns.insert(f.clone(), v.clone());
            }
            for (i, v) in &new_table.indexes {
                diff.added_indexes.insert(i.clone(), v.clone());
            }
            for (e, v) in &new_table.events {
                diff.added_events.insert(e.clone(), v.clone());
            }

            let up_stmts = diff.generate_statements(table_name)?;
            statements.extend(up_stmts);
        }
    }

    // Handle removed tables
    for (table_name, old_table) in &old_snapshot.tables {
        if !new_snapshot.tables.contains_key(table_name) {
            for field in old_table.fields.keys() {
                statements.push(format!("REMOVE FIELD {} ON TABLE {};", field, table_name));
            }
            for idx in old_table.indexes.keys() {
                statements.push(format!("REMOVE INDEX {} ON TABLE {};", idx, table_name));
            }
            for evt in old_table.events.keys() {
                statements.push(format!("REMOVE EVENT {} ON TABLE {};", evt, table_name));
            }
            statements.push(format!("REMOVE TABLE {};", table_name));
        }
    }

    for (edge_name, new_edge) in &new_snapshot.edges {
        if let Some(old_edge) = old_snapshot.edges.get(edge_name) {
            // Build TableDiff
            let diff = EdgeDiff::from_snapshots(old_edge, new_edge)?;
            let up_stmts = diff.generate_statements(edge_name)?;
            statements.extend(up_stmts);
        } else {
            // Table didn't exist before
            let mut diff = EdgeDiff::new(None, Some(new_edge.define_edge_statement.clone()));
            // Populate diff fields etc. if needed, or if fields are already in the `from_snapshots` logic, no need.
            // Actually, since we don't have an old snapshot, we just treat all fields/indexes/events as "added".
            for (f, v) in &new_edge.fields {
                diff.added_columns.insert(f.clone(), v.clone());
            }
            for (i, v) in &new_edge.indexes {
                diff.added_indexes.insert(i.clone(), v.clone());
            }
            for (e, v) in &new_edge.events {
                diff.added_events.insert(e.clone(), v.clone());
            }

            let up_stmts = diff.generate_statements(edge_name)?;
            statements.extend(up_stmts);
        }
    }

    // Handle removed edges
    for (edge_name, old_edge) in &old_snapshot.edges {
        if !new_snapshot.edges.contains_key(edge_name) {
            for field in old_edge.fields.keys() {
                statements.push(format!("REMOVE FIELD {} ON TABLE {};", field, edge_name));
            }
            for idx in old_edge.indexes.keys() {
                statements.push(format!("REMOVE INDEX {} ON TABLE {};", idx, edge_name));
            }
            for evt in old_edge.events.keys() {
                statements.push(format!("REMOVE EVENT {} ON TABLE {};", evt, edge_name));
            }
            statements.push(format!("REMOVE TABLE {};", edge_name));
        }
    }

    Ok(statements)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(get_file_stem(path), expect);
        }
    }
}
