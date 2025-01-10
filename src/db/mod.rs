mod connection;
mod error;
mod post_repository;
mod tag_repository;

pub use connection::Database;
pub use error::DatabaseError;
pub use post_repository::PostRepository;
pub use tag_repository::TagRepository;

// Re-export common types that callers might need
pub use sqlx::SqlitePool;

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use error::DatabaseResult;
    use sqlx::SqlitePool;

    /// Creates a new test database instance with an in-memory SQLite database
    pub async fn create_test_db() -> DatabaseResult<Database> {
        // Use an in-memory database for testing
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        Database::new().await
    }
}
