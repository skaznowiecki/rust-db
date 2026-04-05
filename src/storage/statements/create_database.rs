use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::storage::catalog::catalog_columns;
use crate::storage::constants::{CATALOG_ID, DATA_DIR};
use crate::storage::schema::{self, TableSchema};

pub fn create_database(name: &str) -> Result<(), String> {
    let db_path = format!("{}/{}", DATA_DIR, name);

    if Path::new(&db_path).exists() {
        return Err(format!("Database '{}' already exists", name));
    }

    let catalog_path = format!("{}/{}", db_path, CATALOG_ID);
    fs::create_dir_all(&catalog_path).map_err(|e| format!("Failed to create database: {}", e))?;

    fs::write(format!("{}/data", catalog_path), "")
        .map_err(|e| format!("Failed to create catalog data file: {}", e))?;

    let mut db_schema = HashMap::new();
    db_schema.insert(
        CATALOG_ID.to_string(),
        TableSchema {
            columns: catalog_columns(),
        },
    );
    schema::save(name, &db_schema)?;

    Ok(())
}
