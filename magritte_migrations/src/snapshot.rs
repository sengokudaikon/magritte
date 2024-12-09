use super::Result;
use magritte::SchemaSnapshot;
use std::fs;
use std::path::Path;

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<SchemaSnapshot> {
    let data = fs::read_to_string(path)?;
    let snapshot: SchemaSnapshot = serde_json::from_str(&data)?;
    Ok(snapshot)
}

pub fn save_to_file<P: AsRef<Path>>(s: &SchemaSnapshot, path: P) -> Result<()> {
    let data = serde_json::to_string_pretty(s)?;
    fs::write(path, data)?;
    Ok(())
}
