use std::fs;
use std::io::{BufRead, BufReader};

use crate::constants;
use crate::error::DbError;
use crate::parser::ast::WhereClause;

use super::Table;
use super::filter::match_value;

impl Table {
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
}
