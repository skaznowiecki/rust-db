use crate::error::DbError;
use crate::parser;
use crate::parser::ast::Statement;
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
            Statement::Use(use_db) => {
                let database = Database::load(&use_db.name)?;
                self.db = Some(database);
                Ok(ExecuteResult::DbChanged(use_db.name))
            }
        }
    }
}
