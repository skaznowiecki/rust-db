use crate::parser::ast::{DropDatabase, Statement};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_drop_database(&mut self) -> Result<Statement, String> {
        self.advance(); // consume DATABASE
        let name = self.expect_identifier()?;
        self.consume_optional_semicolon();
        Ok(Statement::DropDatabase(DropDatabase { name }))
    }
}
