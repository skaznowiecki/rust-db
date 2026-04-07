use super::super::parser::Parser;
use crate::error::DbError;
use crate::lexer::token::Token;
use crate::parser::ast::{Operator, Select, Statement, Value, WhereExpr};

impl Parser {
    pub(crate) fn parse_select(&mut self) -> Result<Statement, DbError> {
        self.expect_token(&Token::Asterisk)?;
        self.expect_keyword("FROM")?;
        let table = self.expect_identifier()?;

        let where_clause = if matches!(self.peek(), Some(Token::Keyword(k)) if k == "WHERE") {
            self.advance();
            Some(self.parse_where_expr()?)
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

    fn parse_where_expr(&mut self) -> Result<WhereExpr, DbError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<WhereExpr, DbError> {
        let mut left = self.parse_and()?;

        while matches!(self.peek(), Some(Token::Keyword(k)) if k == "OR") {
            self.advance();
            let right = self.parse_and()?;
            left = WhereExpr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<WhereExpr, DbError> {
        let mut left = self.parse_where_atom()?;

        while matches!(self.peek(), Some(Token::Keyword(k)) if k == "AND") {
            self.advance();
            let right = self.parse_where_atom()?;
            left = WhereExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_where_atom(&mut self) -> Result<WhereExpr, DbError> {
        // Parenthesized expression
        if matches!(self.peek(), Some(Token::LeftParen)) {
            self.advance();
            let expr = self.parse_or()?;
            self.expect_token(&Token::RightParen)?;
            return Ok(expr);
        }

        let column = self.expect_identifier()?;

        // IN (val1, val2, ...)
        if matches!(self.peek(), Some(Token::Keyword(k)) if k == "IN") {
            self.advance();
            self.expect_token(&Token::LeftParen)?;
            let mut values = vec![self.parse_where_value()?];
            while matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
                values.push(self.parse_where_value()?);
            }
            self.expect_token(&Token::RightParen)?;
            return Ok(WhereExpr::In { column, values });
        }

        // BETWEEN low AND high
        if matches!(self.peek(), Some(Token::Keyword(k)) if k == "BETWEEN") {
            self.advance();
            let low = self.parse_where_value()?;
            self.expect_keyword("AND")?;
            let high = self.parse_where_value()?;
            return Ok(WhereExpr::Between { column, low, high });
        }

        // Regular comparison: column OP value
        let operator = self.parse_where_operator()?;
        let value = self.parse_where_value()?;
        Ok(WhereExpr::Comparison { column, operator, value })
    }

    fn parse_where_operator(&mut self) -> Result<Operator, DbError> {
        match self.peek() {
            Some(Token::Equals) => { self.advance(); Ok(Operator::Eq) }
            Some(Token::NotEquals) => { self.advance(); Ok(Operator::NotEq) }
            Some(Token::LessThan) => { self.advance(); Ok(Operator::Lt) }
            Some(Token::LessThanEquals) => { self.advance(); Ok(Operator::Lte) }
            Some(Token::GreaterThan) => { self.advance(); Ok(Operator::Gt) }
            Some(Token::GreaterThanEquals) => { self.advance(); Ok(Operator::Gte) }
            Some(Token::Keyword(k)) if k == "LIKE" => { self.advance(); Ok(Operator::Like) }
            Some(Token::Keyword(k)) if k == "ILIKE" => { self.advance(); Ok(Operator::ILike) }
            Some(token) => Err(DbError::ParseError(format!("Expected operator, got {:?}", token))),
            None => Err(DbError::ParseError("Expected operator, got end of input".into())),
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
                "Expected value, got {:?}", token
            ))),
            None => Err(DbError::ParseError(
                "Expected value, got end of input".into(),
            )),
        }
    }
}
