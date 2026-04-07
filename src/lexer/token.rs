#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(i64),
    StringLiteral(String),
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Asterisk,
}
