use crate::parser;
use crate::parser::ast::Statement;
use crate::storage;

pub enum ExecuteResult {
    Message(String),
    DbChanged(String),
}

pub struct Engine {
    current_db: Option<String>,
}

impl Engine {
    pub fn new() -> Self {
        Self { current_db: None }
    }

    pub fn with_db(db_name: &str) -> Self {
        Self {
            current_db: Some(db_name.to_string()),
        }
    }

    pub fn current_db(&self) -> Option<&str> {
        self.current_db.as_deref()
    }

    pub fn execute(&mut self, sql: &str) -> Result<ExecuteResult, String> {
        let stmt = parser::parse_sql(sql)?;
        self.execute_statement(stmt)
    }

    fn execute_statement(&mut self, stmt: Statement) -> Result<ExecuteResult, String> {
        match stmt {
            Statement::CreateDatabase(create_db) => {
                storage::create_database(&create_db.name)?;
                Ok(ExecuteResult::Message(format!(
                    "Database '{}' created",
                    create_db.name
                )))
            }
            Statement::CreateTable(create_table) => {
                let db = self
                    .current_db
                    .as_deref()
                    .ok_or("No database selected. Use: USE <database_name>")?;
                storage::create_table(db, &create_table)?;
                Ok(ExecuteResult::Message(format!(
                    "Table '{}' created",
                    create_table.name
                )))
            }
            Statement::DropDatabase(drop_db) => {
                storage::delete_database(&drop_db.name)?;
                if self.current_db.as_deref() == Some(&drop_db.name) {
                    self.current_db = None;
                }
                Ok(ExecuteResult::Message(format!(
                    "Database '{}' dropped",
                    drop_db.name
                )))
            }
            Statement::DropTable(drop_table) => {
                let db = self
                    .current_db
                    .as_deref()
                    .ok_or("No database selected. Use: USE <database_name>")?;
                storage::delete_table(db, &drop_table.name)?;
                Ok(ExecuteResult::Message(format!(
                    "Table '{}' dropped",
                    drop_table.name
                )))
            }
            Statement::Use(use_db) => {
                let db_path = format!("./data/{}", use_db.name);
                if !std::path::Path::new(&db_path).exists() {
                    return Err(format!("Database '{}' does not exist", use_db.name));
                }
                self.current_db = Some(use_db.name.clone());
                Ok(ExecuteResult::DbChanged(use_db.name))
            }
        }
    }
}
