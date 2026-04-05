pub mod ast;
pub mod parser;
mod statements;

use crate::error::DbError;
use crate::parser::ast::Statement;

pub fn parse_sql(input: &str) -> Result<Statement, DbError> {
    let tokens = crate::lexer::lexer::tokenize(input)?;
    let mut p = parser::Parser::new(tokens);
    p.parse()
}
