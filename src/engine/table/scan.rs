use std::fs;
use std::io::{BufRead, BufReader};

use crate::constants;
use crate::error::DbError;
use crate::parser::ast::WhereExpr;

use super::Table;
use super::filter::eval_where;

impl Table {
    pub fn scan(&self, where_clause: Option<&WhereExpr>, limit: Option<usize>) -> Result<Vec<Vec<String>>, DbError> {
        let path = constants::table_data_path(&self.db_name, &self.id);

        let file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(DbError::IoError(e.to_string())),
        };

        let mut rows = Vec::new();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            let fields: Vec<String> = line.split('|').map(|s| s.to_string()).collect();

            if let Some(expr) = where_clause {
                if !eval_where(&fields, &|name| self.column_pos(name), expr) {
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
