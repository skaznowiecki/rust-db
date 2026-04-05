use crate::error::DbError;
use crate::lexer::token::Token;
use crate::parser::ast::{
    ColumnDef, CreateTable, DataType, DefaultValue, Statement,
};
use super::super::parser::Parser;

impl Parser {
    pub(crate) fn parse_create_table(&mut self) -> Result<Statement, DbError> {
        self.advance();
        let name = self.expect_identifier()?;
        self.expect_token(&Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            columns.push(self.parse_column_def()?);

            match self.peek() {
                Some(Token::Comma) => { self.advance(); }
                Some(Token::RightParen) => { self.advance(); break; }
                Some(token) => return Err(DbError::ParseError(format!("Expected ',' or ')', got {:?}", token))),
                None => return Err(DbError::ParseError("Unexpected end of input in column list".into())),
            }
        }

        self.consume_optional_semicolon();
        Ok(Statement::CreateTable(CreateTable { name, columns }))
    }

    fn parse_column_def(&mut self) -> Result<ColumnDef, DbError> {
        let name = self.expect_identifier()?;
        let data_type = self.parse_data_type()?;

        let mut col = ColumnDef {
            name,
            data_type,
            is_primary_key: false,
            is_not_null: false,
            is_unique: false,
            default: None,
        };

        loop {
            match self.peek() {
                Some(Token::Comma) | Some(Token::RightParen) | None => break,
                Some(Token::Keyword(k)) if k == "PRIMARY" => {
                    self.advance();
                    self.expect_keyword("KEY")?;
                    col.is_primary_key = true;
                }
                Some(Token::Keyword(k)) if k == "NOT" => {
                    self.advance();
                    self.expect_keyword("NULL")?;
                    col.is_not_null = true;
                }
                Some(Token::Keyword(k)) if k == "UNIQUE" => {
                    self.advance();
                    col.is_unique = true;
                }
                Some(Token::Keyword(k)) if k == "DEFAULT" => {
                    self.advance();
                    col.default = Some(self.parse_default_value()?);
                }
                Some(token) => return Err(DbError::ParseError(format!("Unexpected token in column definition: {:?}", token))),
            }
        }

        Ok(col)
    }

    fn parse_data_type(&mut self) -> Result<DataType, DbError> {
        match self.advance() {
            Some(Token::Keyword(k)) => match k.as_str() {
                "SERIAL" => Ok(DataType::Serial),
                "INTEGER" | "INT" => Ok(DataType::Integer),
                "TEXT" => Ok(DataType::Text),
                "BOOLEAN" => Ok(DataType::Boolean),
                "VARCHAR" => {
                    self.expect_token(&Token::LeftParen)?;
                    let size = match self.advance() {
                        Some(Token::Number(n)) => *n as usize,
                        Some(token) => return Err(DbError::ParseError(format!("Expected number for VARCHAR size, got {:?}", token))),
                        None => return Err(DbError::ParseError("Expected number for VARCHAR size".into())),
                    };
                    self.expect_token(&Token::RightParen)?;
                    Ok(DataType::Varchar(size))
                }
                other => Err(DbError::ParseError(format!("Unknown data type: {}", other))),
            },
            Some(token) => Err(DbError::ParseError(format!("Expected data type, got {:?}", token))),
            None => Err(DbError::ParseError("Expected data type, got end of input".into())),
        }
    }

    fn parse_default_value(&mut self) -> Result<DefaultValue, DbError> {
        match self.advance() {
            Some(Token::Keyword(k)) if k == "TRUE" => Ok(DefaultValue::Bool(true)),
            Some(Token::Keyword(k)) if k == "FALSE" => Ok(DefaultValue::Bool(false)),
            Some(Token::Number(n)) => Ok(DefaultValue::Number(*n)),
            Some(Token::StringLiteral(s)) => Ok(DefaultValue::String(s.clone())),
            Some(token) => Err(DbError::ParseError(format!("Expected default value, got {:?}", token))),
            None => Err(DbError::ParseError("Expected default value, got end of input".into())),
        }
    }
}
