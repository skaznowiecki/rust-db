use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use crate::error::DbError;
use crate::parser::ast::{DataType, DefaultValue, InsertInto, Operator, Value, WhereClause};
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
            if self.column_pos(col_name).is_none() {
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

    pub fn scan(&self, where_clause: Option<&WhereClause>, limit: Option<usize>) -> Result<Vec<Vec<String>>, DbError> {
        let path = constants::table_data_path(&self.db_name, &self.id);

        let file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(DbError::IoError(e.to_string())),
        };

        let col_index = if let Some(wc) = where_clause {
            let idx = self.column_pos(&wc.column)
                .ok_or(DbError::ColumnNotFound {
                    column: wc.column.clone(),
                    table: self.name.clone(),
                })?;
            Some(idx)
        } else {
            None
        };

        let mut rows = Vec::new();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let fields: Vec<String> = line.split('|').map(|s| s.to_string()).collect();

            if let (Some(idx), Some(wc)) = (col_index, where_clause) {
                let field_val = fields.get(idx).map(|s| s.as_str()).unwrap_or("");
                if !match_value(field_val, &wc.operator, &wc.value) {
                    continue;
                }
            }

            rows.push(fields);

            if let Some(lim) = limit {
                if rows.len() >= lim {
                    break;
                }
            }
        }

        Ok(rows)
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

fn match_value(field: &str, operator: &Operator, value: &Value) -> bool {
    match operator {
        Operator::Eq => match_eq(field, value),
        Operator::NotEq => !match_eq(field, value),
        Operator::Lt => match_cmp(field, value, std::cmp::Ordering::Less),
        Operator::Gt => match_cmp(field, value, std::cmp::Ordering::Greater),
        Operator::Like => match_like(field, value, false),
        Operator::ILike => match_like(field, value, true),
    }
}

fn match_eq(field: &str, value: &Value) -> bool {
    match value {
        Value::Number(n) => field == n.to_string(),
        Value::String(s) => field == s,
        Value::Bool(b) => field == b.to_string(),
        Value::Null => field.is_empty(),
    }
}

fn match_cmp(field: &str, value: &Value, expected: std::cmp::Ordering) -> bool {
    match value {
        Value::Number(n) => {
            if let Ok(f) = field.parse::<i64>() {
                f.cmp(n) == expected
            } else {
                false
            }
        }
        Value::String(s) => field.cmp(s.as_str()) == expected,
        _ => false,
    }
}

fn match_like(field: &str, value: &Value, case_insensitive: bool) -> bool {
    let pattern = match value {
        Value::String(s) => s,
        _ => return false,
    };

    let (field, pattern) = if case_insensitive {
        (field.to_lowercase(), pattern.to_lowercase())
    } else {
        (field.to_string(), pattern.clone())
    };

    let starts = pattern.starts_with('%');
    let ends = pattern.ends_with('%');

    match (starts, ends) {
        (true, true) => {
            let inner = &pattern[1..pattern.len() - 1];
            field.contains(inner)
        }
        (true, false) => {
            let suffix = &pattern[1..];
            field.ends_with(suffix)
        }
        (false, true) => {
            let prefix = &pattern[..pattern.len() - 1];
            field.starts_with(prefix)
        }
        (false, false) => field == pattern,
    }
}
