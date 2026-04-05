use std::fs;
use std::path::Path;

use crate::error::DbError;
use crate::storage::constants::{CATALOG_ID, DATA_DIR};
use crate::storage::schema;

pub fn delete_table(db_name: &str, table_name: &str) -> Result<(), DbError> {
    let db_path = format!("{}/{}", DATA_DIR, db_name);

    if !Path::new(&db_path).exists() {
        return Err(DbError::DatabaseNotFound(db_name.into()));
    }

    let catalog_data_path = format!("{}/{}/data", db_path, CATALOG_ID);
    let catalog_content = fs::read_to_string(&catalog_data_path)?;

    let mut table_id: Option<String> = None;
    let mut remaining_lines = Vec::new();

    for line in catalog_content.lines() {
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() >= 2 && fields[1] == table_name {
            table_id = Some(fields[0].to_string());
        } else {
            remaining_lines.push(line);
        }
    }

    let table_id = table_id.ok_or(DbError::TableNotFound(table_name.into()))?;

    let mut new_catalog = remaining_lines.join("\n");
    if !new_catalog.is_empty() {
        new_catalog.push('\n');
    }
    fs::write(&catalog_data_path, &new_catalog)?;

    let table_path = format!("{}/{}", db_path, table_id);
    fs::remove_dir_all(&table_path)?;

    let mut db_schema = schema::load(db_name)?;
    db_schema.remove(&table_id);
    schema::save(db_name, &db_schema)?;

    Ok(())
}
