use thiserror::Error;

/// 数据库操作错误类型
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Record not found")]
    NotFound,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type DbResult<T> = Result<T, DbError>;
