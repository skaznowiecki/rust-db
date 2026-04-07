use crate::error::DbError;
use super::token::Token;

const KEYWORDS: &[&str] = &[
    "CREATE", "DROP", "DATABASE", "TABLE", "USE",
    "INSERT", "INTO", "VALUES",
    "SERIAL", "INTEGER", "INT", "VARCHAR", "TEXT", "BOOLEAN",
    "PRIMARY", "KEY", "NOT", "NULL", "UNIQUE", "DEFAULT",
    "TRUE", "FALSE",
    "SELECT", "FROM", "WHERE", "LIMIT",
    "LIKE", "ILIKE",
    "AND", "OR", "IN", "BETWEEN",
];

pub fn tokenize(input: &str) -> Result<Vec<Token>, DbError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        match ch {
            ';' => { tokens.push(Token::Semicolon); chars.next(); continue; }
            '(' => { tokens.push(Token::LeftParen); chars.next(); continue; }
            ')' => { tokens.push(Token::RightParen); chars.next(); continue; }
            ',' => { tokens.push(Token::Comma); chars.next(); continue; }
            '=' => { tokens.push(Token::Equals); chars.next(); continue; }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::LessThanEquals);
                } else {
                    tokens.push(Token::LessThan);
                }
                continue;
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::GreaterThanEquals);
                } else {
                    tokens.push(Token::GreaterThan);
                }
                continue;
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::NotEquals);
                } else {
                    return Err(DbError::LexerError("Expected '=' after '!'".into()));
                }
                continue;
            }
            '*' => { tokens.push(Token::Asterisk); chars.next(); continue; }
            _ => {}
        }

        if ch == '\'' {
            chars.next();
            let mut s = String::new();
            loop {
                match chars.next() {
                    Some('\'') => break,
                    Some(c) => s.push(c),
                    None => return Err(DbError::LexerError("Unterminated string literal".into())),
                }
            }
            tokens.push(Token::StringLiteral(s));
            continue;
        }

        if ch.is_ascii_digit() {
            let mut num = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let n: i64 = num.parse().map_err(|e| DbError::LexerError(format!("Invalid number: {}", e)))?;
            tokens.push(Token::Number(n));
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' {
            let mut word = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    word.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let upper = word.to_uppercase();
            if KEYWORDS.contains(&upper.as_str()) {
                tokens.push(Token::Keyword(upper));
            } else {
                tokens.push(Token::Identifier(word));
            }
            continue;
        }

        return Err(DbError::LexerError(format!("Unexpected character: '{}'", ch)));
    }

    Ok(tokens)
}
