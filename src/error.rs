use std::fmt;

#[derive(Debug)]
pub enum DbError {
    // Database errors
    DatabaseAlreadyExists(String),
    DatabaseNotFound(String),

    // Table errors
    TableAlreadyExists(String),
    TableNotFound(String),

    // Column errors
    ColumnNotFound { column: String, table: String },
    TypeMismatch { column: String, expected: String, got: String },
    NotNullViolation(String),

    // Parse errors
    ParseError(String),

    // Lexer errors
    LexerError(String),

    // IO errors
    IoError(String),

    // Server errors
    ServerError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::DatabaseAlreadyExists(name) => write!(f, "Database '{}' already exists", name),
            DbError::DatabaseNotFound(name) => write!(f, "Database '{}' does not exist", name),
            DbError::TableAlreadyExists(name) => write!(f, "Table '{}' already exists", name),
            DbError::TableNotFound(name) => write!(f, "Table '{}' does not exist", name),
            DbError::ColumnNotFound { column, table } => {
                write!(f, "Column '{}' does not exist in table '{}'", column, table)
            }
            DbError::TypeMismatch { column, expected, got } => {
                write!(f, "Type mismatch for column '{}': expected {}, got {}", column, expected, got)
            }
            DbError::NotNullViolation(column) => {
                write!(f, "Column '{}' is NOT NULL and has no DEFAULT value", column)
            }
            DbError::ParseError(msg) => write!(f, "{}", msg),
            DbError::LexerError(msg) => write!(f, "{}", msg),
            DbError::IoError(msg) => write!(f, "{}", msg),
            DbError::ServerError(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        DbError::IoError(e.to_string())
    }
}
