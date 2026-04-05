use db::lexer::lexer::tokenize;
use db::lexer::token::Token;

#[test]
fn test_tokenize_create_database() {
    let tokens = tokenize("CREATE DATABASE mi_app;").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Keyword("CREATE".into()),
            Token::Keyword("DATABASE".into()),
            Token::Identifier("mi_app".into()),
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_tokenize_case_insensitive() {
    let tokens = tokenize("create database Test").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Keyword("CREATE".into()),
            Token::Keyword("DATABASE".into()),
            Token::Identifier("Test".into()),
        ]
    );
}

#[test]
fn test_tokenize_create_table() {
    let tokens = tokenize("CREATE TABLE usuarios (id SERIAL PRIMARY KEY, nombre VARCHAR(100) NOT NULL);").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Keyword("CREATE".into()),
            Token::Keyword("TABLE".into()),
            Token::Identifier("usuarios".into()),
            Token::LeftParen,
            Token::Identifier("id".into()),
            Token::Keyword("SERIAL".into()),
            Token::Keyword("PRIMARY".into()),
            Token::Keyword("KEY".into()),
            Token::Comma,
            Token::Identifier("nombre".into()),
            Token::Keyword("VARCHAR".into()),
            Token::LeftParen,
            Token::Number(100),
            Token::RightParen,
            Token::Keyword("NOT".into()),
            Token::Keyword("NULL".into()),
            Token::RightParen,
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_tokenize_default_values() {
    let tokens = tokenize("DEFAULT true").unwrap();
    assert_eq!(
        tokens,
        vec![Token::Keyword("DEFAULT".into()), Token::Keyword("TRUE".into())]
    );
}

#[test]
fn test_tokenize_string_literal() {
    let tokens = tokenize("DEFAULT 'hello'").unwrap();
    assert_eq!(
        tokens,
        vec![Token::Keyword("DEFAULT".into()), Token::StringLiteral("hello".into())]
    );
}

#[test]
fn test_tokenize_number() {
    let tokens = tokenize("DEFAULT 42").unwrap();
    assert_eq!(
        tokens,
        vec![Token::Keyword("DEFAULT".into()), Token::Number(42)]
    );
}

#[test]
fn test_tokenize_unexpected_char() {
    let result = tokenize("CREATE @");
    assert!(result.is_err());
}

#[test]
fn test_tokenize_unterminated_string() {
    let result = tokenize("'unterminated");
    assert!(result.is_err());
}
