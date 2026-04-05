use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::constants::DATA_DIR;
use crate::parser::ast::ColumnDef;

pub type DbSchema = HashMap<String, TableSchema>;

#[derive(Debug, Serialize, Deserialize)]
pub struct TableSchema {
    pub columns: Vec<ColumnDef>,
}

fn schema_path(db_name: &str) -> String {
    format!("{}/{}/schema.json", DATA_DIR, db_name)
}

pub fn load(db_name: &str) -> Result<DbSchema, String> {
    let path = schema_path(db_name);
    if !Path::new(&path).exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read schema: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse schema: {}", e))
}

pub fn save(db_name: &str, schema: &DbSchema) -> Result<(), String> {
    let content = serde_json::to_string_pretty(schema)
        .map_err(|e| format!("Failed to serialize schema: {}", e))?;
    fs::write(schema_path(db_name), content).map_err(|e| format!("Failed to write schema: {}", e))
}
