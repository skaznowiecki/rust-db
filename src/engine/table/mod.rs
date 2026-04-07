mod filter;
mod insert;
mod scan;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

use crate::constants;
use crate::error::DbError;
use crate::parser::ast::{DataType, DefaultValue, Value};
use crate::storage::schema::TableSchema;

pub struct Table {
    pub id: String,
    pub name: String,
    pub schema: TableSchema,
    pub(crate) db_name: String,
    pub(crate) serial_counter: Option<i64>,
    pub(crate) write_buffer: Vec<u8>,
    column_index: HashMap<String, usize>,
}

impl Table {
    fn build_column_index(schema: &TableSchema) -> HashMap<String, usize> {
        schema.columns.iter().enumerate().map(|(i, c)| (c.name.clone(), i)).collect()
    }

    pub fn load(db_name: &str, table_id: &str, table_name: &str, schema: TableSchema) -> Self {
        let serial_counter = Self::find_serial_pos(&schema)
            .map(|pos| Self::read_serial_from_disk(db_name, table_id, pos));
        let column_index = Self::build_column_index(&schema);

        Self {
            id: table_id.to_string(),
            name: table_name.to_string(),
            schema,
            db_name: db_name.to_string(),
            serial_counter,
            write_buffer: Vec::new(),
            column_index,
        }
    }

    pub fn new(db_name: &str, table_id: &str, table_name: &str, schema: TableSchema) -> Self {
        let has_serial = schema
            .columns
            .iter()
            .any(|c| c.data_type == DataType::Serial);
        let column_index = Self::build_column_index(&schema);
        Self {
            id: table_id.to_string(),
            name: table_name.to_string(),
            schema,
            db_name: db_name.to_string(),
            serial_counter: if has_serial { Some(0) } else { None },
            write_buffer: Vec::new(),
            column_index,
        }
    }

    pub fn column_pos(&self, name: &str) -> Option<usize> {
        self.column_index.get(name).copied()
    }

    pub fn flush(&mut self) -> Result<(), DbError> {
        if self.write_buffer.is_empty() {
            return Ok(());
        }

        let path = constants::table_data_path(&self.db_name, &self.id);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(&self.write_buffer)?;
        self.write_buffer.clear();

        Ok(())
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
    }
}

fn default_to_string(default: &DefaultValue) -> String {
    match default {
        DefaultValue::Number(n) => n.to_string(),
        DefaultValue::String(s) => s.clone(),
        DefaultValue::Bool(b) => b.to_string(),
    }
}
