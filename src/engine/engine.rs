use crate::error::DbError;
use crate::parser;
use crate::parser::ast::{SelectColumns, Statement};
use crate::storage;

use super::database::Database;

pub enum ExecuteResult {
    Message(String),
    DbChanged(String),
}

pub struct Engine {
    db: Option<Database>,
}

impl Engine {
    pub fn new() -> Self {
        Self { db: None }
    }

    pub fn with_db(db_name: &str) -> Self {
        Self {
            db: Database::load(db_name).ok(),
        }
    }

    pub fn current_db(&self) -> Option<&str> {
        self.db.as_ref().map(|db| db.name.as_str())
    }

    fn require_db(&mut self) -> Result<&mut Database, DbError> {
        self.db
            .as_mut()
            .ok_or(DbError::DatabaseNotFound("No database selected".into()))
    }

    pub fn flush(&mut self) -> Result<(), DbError> {
        if let Some(ref mut db) = self.db {
            db.flush_all()?;
        }
        Ok(())
    }

    pub fn execute(&mut self, sql: &str) -> Result<ExecuteResult, DbError> {
        let stmt = parser::parse_sql(sql)?;
        self.execute_statement(stmt)
    }

    fn execute_statement(&mut self, stmt: Statement) -> Result<ExecuteResult, DbError> {
        match stmt {
            Statement::CreateDatabase(create_db) => {
                storage::create_database(&create_db.name)?;
                Ok(ExecuteResult::Message(format!(
                    "Database '{}' created",
                    create_db.name
                )))
            }
            Statement::CreateTable(create_table) => {
                let db = self.require_db()?;
                let msg = db.create_table(&create_table)?;
                Ok(ExecuteResult::Message(msg))
            }
            Statement::DropDatabase(drop_db) => {
                storage::delete_database(&drop_db.name)?;
                if self.current_db() == Some(&drop_db.name) {
                    self.db = None;
                }
                Ok(ExecuteResult::Message(format!(
                    "Database '{}' dropped",
                    drop_db.name
                )))
            }
            Statement::DropTable(drop_table) => {
                let db = self.require_db()?;
                let msg = db.drop_table(&drop_table.name)?;
                Ok(ExecuteResult::Message(msg))
            }
            Statement::InsertInto(insert) => {
                let db = self.require_db()?;
                let msg = db.insert(&insert)?;
                Ok(ExecuteResult::Message(msg))
            }
            Statement::Select(select) => {
                let db = self.require_db()?;
                let (headers, rows) = db.select(&select.table, select.where_clause.as_ref(), select.limit)?;

                let (headers, rows) = match &select.columns {
                    SelectColumns::All => (headers, rows),
                    SelectColumns::Columns(cols) => {
                        let indices: Vec<usize> = cols.iter().map(|c| {
                            headers.iter().position(|h| h == c)
                                .ok_or(DbError::ColumnNotFound {
                                    column: c.clone(),
                                    table: select.table.clone(),
                                })
                        }).collect::<Result<Vec<_>, _>>()?;

                        let proj_headers = indices.iter().map(|&i| headers[i].clone()).collect();
                        let proj_rows = rows.iter().map(|row| {
                            indices.iter().map(|&i| row[i].clone()).collect()
                        }).collect();

                        (proj_headers, proj_rows)
                    }
                };

                let output = format_table(&headers, &rows);
                Ok(ExecuteResult::Message(output))
            }
            Statement::Use(use_db) => {
                let database = Database::load(&use_db.name)?;
                self.db = Some(database);
                Ok(ExecuteResult::DbChanged(use_db.name))
            }
            Statement::ShowDatabases(_) => {
                let databases = storage::list_databases()?;
                let headers = vec!["Database".to_string()];
                let rows: Vec<Vec<String>> = databases.iter().map(|d| vec![d.clone()]).collect();
                let output = format_table(&headers, &rows);
                Ok(ExecuteResult::Message(output))
            }
            Statement::ShowTables(_) => {
                let db = self.require_db()?;
                let tables = storage::list_tables(&db.name)?;
                let headers = vec!["Table".to_string()];
                let rows: Vec<Vec<String>> = tables.iter().map(|t| vec![t.clone()]).collect();
                let output = format_table(&headers, &rows);
                Ok(ExecuteResult::Message(output))
            }
        }
    }
}

fn format_table(headers: &[String], rows: &[Vec<String>]) -> String {
    let col_count = headers.len();

    // Calculate max width per column
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, val) in row.iter().enumerate() {
            if i < col_count && val.len() > widths[i] {
                widths[i] = val.len();
            }
        }
    }

    // Build separator line: +----+------+
    let separator: String = widths
        .iter()
        .map(|w| format!("-{}-", "-".repeat(*w)))
        .collect::<Vec<_>>()
        .join("+");
    let separator = format!("+{}+", separator);

    // Build a row line: | val | val |
    let format_row = |values: &[String]| -> String {
        let cells: Vec<String> = values
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < col_count)
            .map(|(i, v)| format!(" {:<width$} ", v, width = widths[i]))
            .collect();
        format!("|{}|", cells.join("|"))
    };

    let mut output = String::new();
    output.push_str(&separator);
    output.push('\n');
    output.push_str(&format_row(headers));
    output.push('\n');
    output.push_str(&separator);
    for row in rows {
        output.push('\n');
        output.push_str(&format_row(row));
    }
    output.push('\n');
    output.push_str(&separator);
    output.push('\n');
    output.push_str(&format!("({} rows)", rows.len()));
    output
}
