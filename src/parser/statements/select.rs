use super::super::parser::Parser;
use crate::error::DbError;
use crate::lexer::token::Token;
use crate::parser::ast::{Operator, Select, Statement, Value, WhereClause};

impl Parser {
    pub(crate) fn parse_select(&mut self) -> Result<Statement, DbError> {
        self.expect_token(&Token::Asterisk)?;
        self.expect_keyword("FROM")?;
        let table = self.expect_identifier()?;

        let where_clause = if matches!(self.peek(), Some(Token::Keyword(k)) if k == "WHERE") {
            self.advance();
            let column = self.expect_identifier()?;
            let operator = self.parse_where_operator()?;
            let value = self.parse_where_value()?;
            Some(WhereClause {
                column,
                operator,
                value,
            })
        } else {
            None
        };

        let limit = if matches!(self.peek(), Some(Token::Keyword(k)) if k == "LIMIT") {
            self.advance();
            match self.advance() {
                Some(Token::Number(n)) => Some(*n as usize),
                Some(token) => {
                    return Err(DbError::ParseError(format!(
                        "Expected number after LIMIT, got {:?}",
                        token
                    )));
                }
                None => {
                    return Err(DbError::ParseError(
                        "Expected number after LIMIT, got end of input".into(),
                    ));
                }
            }
        } else {
            None
        };

        self.consume_optional_semicolon();

        if let Some(token) = self.peek() {
            return Err(DbError::ParseError(format!(
                "Unexpected token: {:?}",
                token
            )));
        }

        Ok(Statement::Select(Select {
            table,
            where_clause,
            limit,
        }))
    }

    fn parse_where_operator(&mut self) -> Result<Operator, DbError> {
        match self.peek() {
            Some(Token::Equals) => {
                self.advance();
                Ok(Operator::Eq)
            }
            Some(Token::NotEquals) => {
                self.advance();
                Ok(Operator::NotEq)
            }
            Some(Token::LessThan) => {
                self.advance();
                Ok(Operator::Lt)
            }
            Some(Token::GreaterThan) => {
                self.advance();
                Ok(Operator::Gt)
            }
            Some(Token::Keyword(k)) if k == "LIKE" => {
                self.advance();
                Ok(Operator::Like)
            }
            Some(Token::Keyword(k)) if k == "ILIKE" => {
                self.advance();
                Ok(Operator::ILike)
            }
            Some(token) => Err(DbError::ParseError(format!(
                "Expected operator, got {:?}",
                token
            ))),
            None => Err(DbError::ParseError(
                "Expected operator, got end of input".into(),
            )),
        }
    }

    fn parse_where_value(&mut self) -> Result<Value, DbError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Value::Number(*n)),
            Some(Token::StringLiteral(s)) => Ok(Value::String(s.clone())),
            Some(Token::Keyword(k)) if k == "TRUE" => Ok(Value::Bool(true)),
            Some(Token::Keyword(k)) if k == "FALSE" => Ok(Value::Bool(false)),
            Some(Token::Keyword(k)) if k == "NULL" => Ok(Value::Null),
            Some(token) => Err(DbError::ParseError(format!(
                "Expected value after operator, got {:?}",
                token
            ))),
            None => Err(DbError::ParseError(
                "Expected value after operator, got end of input".into(),
            )),
        }
    }
}
