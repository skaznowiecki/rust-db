use crate::lexer::token::Token;
use super::ast::Statement;

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    pub(crate) fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    pub(crate) fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        match self.advance() {
            Some(Token::Keyword(k)) if k == keyword => Ok(()),
            Some(token) => Err(format!("Expected keyword '{}', got {:?}", keyword, token)),
            None => Err(format!("Expected keyword '{}', got end of input", keyword)),
        }
    }

    pub(crate) fn expect_identifier(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Identifier(name)) => Ok(name.clone()),
            Some(token) => Err(format!("Expected identifier, got {:?}", token)),
            None => Err("Expected identifier, got end of input".into()),
        }
    }

    pub(crate) fn expect_token(&mut self, expected: &Token) -> Result<(), String> {
        match self.advance() {
            Some(token) if token == expected => Ok(()),
            Some(token) => Err(format!("Expected {:?}, got {:?}", expected, token)),
            None => Err(format!("Expected {:?}, got end of input", expected)),
        }
    }

    pub(crate) fn consume_optional_semicolon(&mut self) {
        if let Some(Token::Semicolon) = self.peek() {
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Statement, String> {
        match self.peek() {
            Some(Token::Keyword(k)) if k == "CREATE" => {
                self.advance();
                match self.peek() {
                    Some(Token::Keyword(k)) if k == "DATABASE" => self.parse_create_database(),
                    Some(Token::Keyword(k)) if k == "TABLE" => self.parse_create_table(),
                    Some(token) => Err(format!("Unexpected token after CREATE: {:?}", token)),
                    None => Err("Unexpected end of input after CREATE".into()),
                }
            }
            Some(Token::Keyword(k)) if k == "DROP" => {
                self.advance();
                match self.peek() {
                    Some(Token::Keyword(k)) if k == "DATABASE" => self.parse_drop_database(),
                    Some(Token::Keyword(k)) if k == "TABLE" => self.parse_drop_table(),
                    Some(token) => Err(format!("Unexpected token after DROP: {:?}", token)),
                    None => Err("Unexpected end of input after DROP".into()),
                }
            }
            Some(Token::Keyword(k)) if k == "USE" => self.parse_use(),
            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            None => Err("Empty input".into()),
        }
    }
}
