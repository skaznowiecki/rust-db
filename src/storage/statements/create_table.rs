use std::fs;
use std::path::Path;

use crate::error::DbError;
use crate::parser::ast::CreateTable;
use crate::storage::catalog::{find_by_name, parse_catalog};
use crate::constants::{self, FIRST_TABLE_ID};
use crate::storage::file_utils::read_last_line;
use crate::storage::schema;

pub fn create_table(db_name: &str, table: &CreateTable) -> Result<(), DbError> {
    if !Path::new(&constants::db_path(db_name)).exists() {
        return Err(DbError::DatabaseNotFound(db_name.into()));
    }

    let catalog_path = constants::catalog_data_path(db_name);
    let catalog_content = fs::read_to_string(&catalog_path)?;
    let entries = parse_catalog(&catalog_content);

    if find_by_name(&entries, &table.name).is_some() {
        return Err(DbError::TableAlreadyExists(table.name.clone()));
    }

    let next_id = match read_last_line(&catalog_path)? {
        None => FIRST_TABLE_ID,
        Some(last_line) => {
            let last_id = last_line
                .split('|')
                .next()
                .ok_or(DbError::IoError("Invalid catalog entry".into()))?
                .parse::<u64>()
                .map_err(|e| DbError::IoError(format!("Invalid ID in catalog: {}", e)))?;
            last_id + 1
        }
    };

    let catalog_record = format!("{}|{}|default\n", next_id, table.name);
    let mut catalog_data = catalog_content;
    catalog_data.push_str(&catalog_record);
    fs::write(&catalog_path, &catalog_data)?;

    let table_id = next_id.to_string();
    let table_dir = constants::table_dir_path(db_name, &table_id);
    fs::create_dir_all(&table_dir)?;
    fs::write(constants::table_data_path(db_name, &table_id), "")?;

    schema::add_table(db_name, &table_id, table.columns.clone())?;

    Ok(())
}
