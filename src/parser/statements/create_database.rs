use crate::error::DbError;
use crate::parser::ast::{CreateDatabase, Statement};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_create_database(&mut self) -> Result<Statement, DbError> {
        self.advance();
        let name = self.expect_identifier()?;
        self.consume_optional_semicolon();
        Ok(Statement::CreateDatabase(CreateDatabase { name }))
    }
}
