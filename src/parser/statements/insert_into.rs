use crate::error::DbError;
use crate::lexer::token::Token;
use crate::parser::ast::{InsertInto, Statement, Value};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_insert_into(&mut self) -> Result<Statement, DbError> {
        self.expect_keyword("INTO")?;
        let table = self.expect_identifier()?;

        self.expect_token(&Token::LeftParen)?;
        let mut columns = Vec::new();
        loop {
            columns.push(self.expect_identifier()?);
            match self.peek() {
                Some(Token::Comma) => { self.advance(); }
                Some(Token::RightParen) => { self.advance(); break; }
                Some(token) => return Err(DbError::ParseError(format!("Expected ',' or ')' in column list, got {:?}", token))),
                None => return Err(DbError::ParseError("Unexpected end of input in column list".into())),
            }
        }

        self.expect_keyword("VALUES")?;

        self.expect_token(&Token::LeftParen)?;
        let mut values = Vec::new();
        loop {
            values.push(self.parse_value()?);
            match self.peek() {
                Some(Token::Comma) => { self.advance(); }
                Some(Token::RightParen) => { self.advance(); break; }
                Some(token) => return Err(DbError::ParseError(format!("Expected ',' or ')' in value list, got {:?}", token))),
                None => return Err(DbError::ParseError("Unexpected end of input in value list".into())),
            }
        }

        if columns.len() != values.len() {
            return Err(DbError::ParseError(format!(
                "Column count ({}) does not match value count ({})",
                columns.len(),
                values.len()
            )));
        }

        self.consume_optional_semicolon();
        Ok(Statement::InsertInto(InsertInto { table, columns, values }))
    }

    fn parse_value(&mut self) -> Result<Value, DbError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Value::Number(*n)),
            Some(Token::StringLiteral(s)) => Ok(Value::String(s.clone())),
            Some(Token::Keyword(k)) if k == "TRUE" => Ok(Value::Bool(true)),
            Some(Token::Keyword(k)) if k == "FALSE" => Ok(Value::Bool(false)),
            Some(Token::Keyword(k)) if k == "NULL" => Ok(Value::Null),
            Some(token) => Err(DbError::ParseError(format!("Expected value, got {:?}", token))),
            None => Err(DbError::ParseError("Expected value, got end of input".into())),
        }
    }
}
