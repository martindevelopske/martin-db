use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Table '{0}' already exists")]
    TableAlreadyExists(String),

    #[error("Table '{0}' not found")]
    TableNotFound(String),

    #[error("Column '{0}' not found")]
    ColumnNotFound(String),

    #[error("Unique constraint violation on column '{0}'")]
    UniqueViolation(String),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("IO Error: {0}")]
    IoError(String),
}
