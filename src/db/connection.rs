use std::env;
use dotenv::dotenv;
use sqlx::SqlitePool;

use super::{error::DatabaseResult, DatabaseError, PostRepository, TagRepository};

/// Main database interface that provides access to all repositories
#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
    posts: PostRepository,
    tags: TagRepository,
}

impl Database {
    /// Creates a new Database instance, establishing the connection pool
    /// and running any pending migrations
    pub async fn new() -> DatabaseResult<Self> {
        // Load environment variables
        dotenv().ok();
        let db_url = env::var("DATABASE_URL")
            .map_err(|_| DatabaseError::Configuration("DATABASE_URL must be set".to_string()))?;

        // Create connection pool
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(DatabaseError::Sqlx)?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(DatabaseError::Migration)?;

        // Initialize repositories

        let tags = TagRepository::new(pool.clone());
        let posts = PostRepository::new(pool.clone());

        Ok(Self { pool, posts, tags })
    }

    /// Provides access to post-related operations
    pub fn posts(&self) -> &PostRepository {
        &self.posts
    }

    /// Provides access to tag-related operations
    pub fn tags(&self) -> &TagRepository {
        &self.tags
    }

    /// Provides direct access to the connection pool if needed
    /// Note: Prefer using the repository methods instead of direct pool access
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Creates a new transaction that can be used across repositories
    pub async fn transaction(&self) -> DatabaseResult<sqlx::Transaction<'static, sqlx::Sqlite>> {
        self.pool
            .begin()
            .await
            .map_err(|e| DatabaseError::Transaction(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Once};

    pub async fn create_test_db() -> DatabaseResult<Database> {
        // Use an in-memory database for testing
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        Database::new().await
    }

    // Helper function to set up test environment
    fn setup_test_env() {
        env::set_var("DATABASE_URL", "sqlite::memory:");
    }

    #[tokio::test]
    async fn test_new_database_connection() {
        setup_test_env();
        let db = Database::new().await;
        assert!(db.is_ok(), "Should successfully create database connection");
    }


    #[tokio::test]
    async fn test_repository_access() {
        setup_test_env();
        let db = Database::new().await.unwrap();
        
        // Test posts repository access
        let posts_repo = db.posts();
        assert!(!std::ptr::eq(posts_repo, &PostRepository::new(db.pool().clone())),
                "Should return reference to existing repository");

        // Test tags repository access
        let tags_repo = db.tags();
        assert!(!std::ptr::eq(tags_repo, &TagRepository::new(db.pool().clone())),
                "Should return reference to existing repository");
    }

    #[tokio::test]
    async fn test_transaction_creation() {
        setup_test_env();
        let db = Database::new().await.unwrap();
        
        let transaction = db.transaction().await;
        assert!(transaction.is_ok(), "Should successfully create transaction");
    }

    #[tokio::test]
    async fn test_pool_access() {
        setup_test_env();
        let db = Database::new().await.unwrap();
        
        let pool = db.pool();
        assert!(pool.acquire().await.is_ok(), "Pool should be functional");
    }

}
