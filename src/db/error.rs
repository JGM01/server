use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Transaction error: {0}")]
    Transaction(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

// Helper functions remain the same
impl DatabaseError {
    pub fn not_found(resource: &str, identifier: &str) -> Self {
        Self::NotFound(format!("{} with identifier '{}' not found", resource, identifier))
    }

    pub fn duplicate(resource: &str, identifier: &str) -> Self {
        Self::DuplicateEntry(format!("{} with identifier '{}' already exists", resource, identifier))
    }
    
    pub fn validation(message: &str) -> Self {
        Self::Validation(message.to_string())
    }
}
