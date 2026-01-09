pub mod engine;
pub mod error;
pub mod parser;
pub mod storage;

pub use error::db_error::DbError;

pub type Result<T, E = DbError> = std::result::Result<T, E>;
