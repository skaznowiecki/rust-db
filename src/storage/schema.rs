use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::DbError;
use crate::constants::DATA_DIR;
use crate::parser::ast::ColumnDef;

pub type DbSchema = HashMap<String, TableSchema>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub columns: Vec<ColumnDef>,
}

fn schema_path(db_name: &str) -> String {
    format!("{}/{}/schema.json", DATA_DIR, db_name)
}

pub fn load(db_name: &str) -> Result<DbSchema, DbError> {
    let path = schema_path(db_name);
    if !Path::new(&path).exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(&path)?;
    serde_json::from_str(&content).map_err(|e| DbError::IoError(format!("Failed to parse schema: {}", e)))
}

pub fn create_initial(db_name: &str) -> Result<(), DbError> {
    use crate::constants::CATALOG_ID;
    use crate::storage::catalog::catalog_columns;

    let mut db_schema = HashMap::new();
    db_schema.insert(
        CATALOG_ID.to_string(),
        TableSchema {
            columns: catalog_columns(),
        },
    );
    save(db_name, &db_schema)
}

pub fn add_table(db_name: &str, table_id: &str, columns: Vec<ColumnDef>) -> Result<(), DbError> {
    let mut db_schema = load(db_name)?;
    db_schema.insert(
        table_id.to_string(),
        TableSchema { columns },
    );
    save(db_name, &db_schema)
}

pub fn remove_table(db_name: &str, table_id: &str) -> Result<(), DbError> {
    let mut db_schema = load(db_name)?;
    db_schema.remove(table_id);
    save(db_name, &db_schema)
}

fn save(db_name: &str, schema: &DbSchema) -> Result<(), DbError> {
    let content = serde_json::to_string_pretty(schema)
        .map_err(|e| DbError::IoError(format!("Failed to serialize schema: {}", e)))?;
    fs::write(schema_path(db_name), content)?;
    Ok(())
}
