use std::fs;
use std::path::Path;

use crate::storage::constants::DATA_DIR;

pub fn delete_database(name: &str) -> Result<(), String> {
    let db_path = format!("{}/{}", DATA_DIR, name);

    if !Path::new(&db_path).exists() {
        return Err(format!("Database '{}' does not exist", name));
    }

    fs::remove_dir_all(&db_path)
        .map_err(|e| format!("Failed to delete database: {}", e))?;

    Ok(())
}
