use crate::error::DbError;
use crate::parser::ast::{ShowTables, Statement};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_show_tables(&mut self) -> Result<Statement, DbError> {
        self.advance();
        self.consume_optional_semicolon();
        Ok(Statement::ShowTables(ShowTables))
    }
}
