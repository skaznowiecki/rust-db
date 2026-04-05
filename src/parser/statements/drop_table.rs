use crate::parser::ast::{DropTable, Statement};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_drop_table(&mut self) -> Result<Statement, String> {
        self.advance(); // consume TABLE
        let name = self.expect_identifier()?;
        self.consume_optional_semicolon();
        Ok(Statement::DropTable(DropTable { name }))
    }
}
