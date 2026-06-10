use std::fs;
use std::path::Path;

use crate::constants;
use crate::error::DbError;

pub fn list_databases() -> Result<Vec<String>, DbError> {
    let data_dir = Path::new(constants::DATA_DIR);
    if !data_dir.exists() {
        return Ok(vec![]);
    }

    let mut databases = Vec::new();
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let schema_path = format!("{}/{}/schema.json", constants::DATA_DIR, name);
        if Path::new(&schema_path).exists() {
            databases.push(name);
        }
    }

    databases.sort();
    Ok(databases)
}
