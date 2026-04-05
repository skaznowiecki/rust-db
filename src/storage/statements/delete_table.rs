use std::fs;
use std::path::Path;

use crate::error::DbError;
use crate::storage::catalog::{find_by_name, parse_catalog};
use crate::constants;
use crate::storage::schema;

pub fn delete_table(db_name: &str, table_name: &str) -> Result<(), DbError> {
    if !Path::new(&constants::db_path(db_name)).exists() {
        return Err(DbError::DatabaseNotFound(db_name.into()));
    }

    let catalog_path = constants::catalog_data_path(db_name);
    let catalog_content = fs::read_to_string(&catalog_path)?;
    let entries = parse_catalog(&catalog_content);

    let entry = find_by_name(&entries, table_name)
        .ok_or(DbError::TableNotFound(table_name.into()))?;
    let table_id = entry.id.clone();

    let new_content: String = entries
        .iter()
        .filter(|e| e.name != table_name)
        .map(|e| format!("{}|{}|{}", e.id, e.name, e.engine))
        .collect::<Vec<_>>()
        .join("\n");
    let new_catalog = if new_content.is_empty() {
        String::new()
    } else {
        format!("{}\n", new_content)
    };
    fs::write(&catalog_path, &new_catalog)?;

    fs::remove_dir_all(constants::table_dir_path(db_name, &table_id))?;

    schema::remove_table(db_name, &table_id)?;

    Ok(())
}
