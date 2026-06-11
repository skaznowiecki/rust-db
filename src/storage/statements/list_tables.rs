use std::fs;
use std::path::Path;

use crate::constants;
use crate::error::DbError;
use crate::storage::catalog::parse_catalog;

pub fn list_tables(db_name: &str) -> Result<Vec<String>, DbError> {
    let catalog_path = constants::catalog_data_path(db_name);
    if !Path::new(&catalog_path).exists() {
        return Err(DbError::DatabaseNotFound(db_name.into()));
    }

    let catalog_content = fs::read_to_string(&catalog_path)?;
    let mut tables: Vec<String> = parse_catalog(&catalog_content)
        .into_iter()
        .map(|entry| entry.name)
        .collect();
    tables.sort();
    Ok(tables)
}
