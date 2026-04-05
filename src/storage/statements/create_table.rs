use std::fs;
use std::path::Path;

use crate::parser::ast::CreateTable;
use crate::storage::constants::{CATALOG_ID, DATA_DIR, FIRST_TABLE_ID};
use crate::storage::file_utils::read_last_line;
use crate::storage::schema::{self, TableSchema};

pub fn create_table(db_name: &str, table: &CreateTable) -> Result<(), String> {
    let db_path = format!("{}/{}", DATA_DIR, db_name);

    if !Path::new(&db_path).exists() {
        return Err(format!("Database '{}' does not exist", db_name));
    }

    let mut db_schema = schema::load(db_name)?;

    let catalog_data_path = format!("{}/{}/data", db_path, CATALOG_ID);
    let catalog_content = fs::read_to_string(&catalog_data_path)
        .map_err(|e| format!("Failed to read catalog: {}", e))?;

    for line in catalog_content.lines() {
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() >= 2 && fields[1] == table.name {
            return Err(format!("Table '{}' already exists", table.name));
        }
    }

    let next_id = match read_last_line(&catalog_data_path)? {
        None => FIRST_TABLE_ID,
        Some(last_line) => {
            let last_id = last_line
                .split('|')
                .next()
                .ok_or("Invalid catalog entry")?
                .parse::<u64>()
                .map_err(|e| format!("Invalid ID in catalog: {}", e))?;
            last_id + 1
        }
    };

    let catalog_record = format!("{}|{}|default\n", next_id, table.name);
    let mut catalog_data = catalog_content;
    catalog_data.push_str(&catalog_record);
    fs::write(&catalog_data_path, &catalog_data)
        .map_err(|e| format!("Failed to update catalog: {}", e))?;

    let table_path = format!("{}/{}", db_path, next_id);
    fs::create_dir_all(&table_path)
        .map_err(|e| format!("Failed to create table directory: {}", e))?;
    fs::write(format!("{}/data", table_path), "")
        .map_err(|e| format!("Failed to create table data file: {}", e))?;

    db_schema.insert(
        next_id.to_string(),
        TableSchema {
            columns: table.columns.clone(),
        },
    );
    schema::save(db_name, &db_schema)?;

    Ok(())
}
