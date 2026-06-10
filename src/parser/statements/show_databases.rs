use crate::error::DbError;
use crate::parser::ast::{ShowDatabases, Statement};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_show_databases(&mut self) -> Result<Statement, DbError> {
        self.advance();
        self.consume_optional_semicolon();
        Ok(Statement::ShowDatabases(ShowDatabases))
    }
}
