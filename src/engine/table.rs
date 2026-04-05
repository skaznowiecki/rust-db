use std::fs::OpenOptions;
use std::io::Write;

use crate::error::DbError;
use crate::parser::ast::{DataType, DefaultValue, InsertInto, Value};
use crate::constants::{self, BUFFER_SIZE};
use crate::storage::file_utils::read_last_line;
use crate::storage::schema::TableSchema;

pub struct Table {
    pub id: String,
    pub name: String,
    pub schema: TableSchema,
    db_name: String,
    serial_counter: Option<i64>,
    write_buffer: Vec<u8>,
}

impl Table {
    pub fn load(db_name: &str, table_id: &str, table_name: &str, schema: TableSchema) -> Self {
        let serial_counter = Self::find_serial_pos(&schema)
            .map(|pos| Self::read_serial_from_disk(db_name, table_id, pos));

        Self {
            id: table_id.to_string(),
            name: table_name.to_string(),
            schema,
            db_name: db_name.to_string(),
            serial_counter,
            write_buffer: Vec::new(),
        }
    }

    pub fn new(db_name: &str, table_id: &str, table_name: &str, schema: TableSchema) -> Self {
        let has_serial = schema
            .columns
            .iter()
            .any(|c| c.data_type == DataType::Serial);
        Self {
            id: table_id.to_string(),
            name: table_name.to_string(),
            schema,
            db_name: db_name.to_string(),
            serial_counter: if has_serial { Some(0) } else { None },
            write_buffer: Vec::new(),
        }
    }

    fn find_serial_pos(schema: &TableSchema) -> Option<usize> {
        schema
            .columns
            .iter()
            .position(|c| c.data_type == DataType::Serial)
    }

    fn read_serial_from_disk(db_name: &str, table_id: &str, serial_pos: usize) -> i64 {
        let path = constants::table_data_path(db_name, table_id);
        match read_last_line(&path) {
            Ok(Some(line)) => {
                let field = line.split('|').nth(serial_pos).unwrap_or("0");
                field.parse::<i64>().unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn next_serial(&mut self) -> i64 {
        let db_name = self.db_name.clone();
        let table_id = self.id.clone();
        let schema = &self.schema;

        let counter = self.serial_counter.get_or_insert_with(|| {
            let pos = Self::find_serial_pos(schema).unwrap_or(0);
            Self::read_serial_from_disk(&db_name, &table_id, pos)
        });
        *counter += 1;
        *counter
    }

    fn validate_columns(&self, columns: &[String]) -> Result<(), DbError> {
        for col_name in columns {
            if !self.schema.columns.iter().any(|c| &c.name == col_name) {
                return Err(DbError::ColumnNotFound {
                    column: col_name.clone(),
                    table: self.name.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_types(&self, columns: &[String], values: &[Value]) -> Result<(), DbError> {
        for (i, col_name) in columns.iter().enumerate() {
            let col_def = self
                .schema
                .columns
                .iter()
                .find(|c| &c.name == col_name)
                .unwrap();
            match (&values[i], &col_def.data_type) {
                (Value::Number(_), DataType::Integer) => {}
                (Value::Number(_), DataType::Serial) => {}
                (Value::String(_), DataType::Varchar(_)) => {}
                (Value::String(_), DataType::Text) => {}
                (Value::Bool(_), DataType::Boolean) => {}
                (Value::Null, _) => {}
                _ => {
                    return Err(DbError::TypeMismatch {
                        column: col_name.clone(),
                        expected: format!("{:?}", col_def.data_type),
                        got: format!("{:?}", values[i]),
                    });
                }
            }
        }
        Ok(())
    }

    fn build_row(&mut self, insert: &InsertInto) -> Result<String, DbError> {
        let serial_val = Self::find_serial_pos(&self.schema).and_then(|_| {
            let serial_col = self
                .schema
                .columns
                .iter()
                .find(|c| c.data_type == DataType::Serial)
                .unwrap();
            if insert.columns.contains(&serial_col.name) {
                None
            } else {
                Some(self.next_serial())
            }
        });

        let mut row_values: Vec<String> = Vec::new();

        for col_def in &self.schema.columns {
            let provided_idx = insert.columns.iter().position(|c| c == &col_def.name);

            if let Some(idx) = provided_idx {
                row_values.push(value_to_string(&insert.values[idx]));
            } else if col_def.data_type == DataType::Serial {
                row_values.push(serial_val.unwrap().to_string());
            } else if let Some(ref default) = col_def.default {
                row_values.push(default_to_string(default));
            } else if col_def.is_not_null {
                return Err(DbError::NotNullViolation(col_def.name.clone()));
            } else {
                row_values.push(String::new());
            }
        }

        Ok(row_values.join("|"))
    }

    pub fn insert(&mut self, insert: &InsertInto) -> Result<String, DbError> {
        self.validate_columns(&insert.columns)?;
        self.validate_types(&insert.columns, &insert.values)?;

        let row = self.build_row(insert)?;
        self.write_buffer.extend_from_slice(row.as_bytes());
        self.write_buffer.push(b'\n');

        if self.write_buffer.len() >= BUFFER_SIZE {
            self.flush()?;
        }

        Ok(format!("1 row inserted into '{}'", self.name))
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
