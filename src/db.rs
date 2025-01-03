use std::env;

use dotenv::dotenv;
use sqlx::SqlitePool;

pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Load the databse url environment variable (in .env)
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("Database URL must be set!");

        // Create the connection pool
        let pool = SqlitePool::connect(&db_url).await?;

        // Run migration :D
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Error;

    // Helper function to create a clean database instance for each test
    async fn setup_test_db() -> Database {
        // Use an in-memory database for testing to ensure isolation
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        Database::new()
            .await
            .expect("Failed to create test database")
    }

    #[sqlx::test]
    async fn test_create_tag_success() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;
        let tag_name = "rust";

        // Act
        let result = sqlx::query!("INSERT INTO tags (name) VALUES (?)", tag_name)
            .execute(&db.pool)
            .await?;

        // Assert
        assert_eq!(result.rows_affected(), 1);
        Ok(())
    }

    #[sqlx::test]
    async fn test_create_duplicate_tag_fails() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;
        let tag_name = "rust";

        // First insertion succeeds
        sqlx::query!("INSERT INTO tags (name) VALUES (?)", tag_name)
            .execute(&db.pool)
            .await?;

        // Act
        let result = sqlx::query!("INSERT INTO tags (name) VALUES (?)", tag_name)
            .execute(&db.pool)
            .await;

        // Assert
        assert!(matches!(result, Err(Error::Database(_))));
        Ok(())
    }

    #[sqlx::test]
    async fn test_post_creation_with_valid_type() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;
        let valid_type = "blog";

        // Act
        let result = sqlx::query!(
            r#"
            INSERT INTO posts (type, title, slug, content, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
            valid_type,
            "Test Title",
            "test-slug",
            "Test content",
            "Test description"
        )
        .execute(&db.pool)
        .await?;

        // Assert
        assert_eq!(result.rows_affected(), 1);
        Ok(())
    }

    #[sqlx::test]
    async fn test_post_creation_with_invalid_type() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;
        let invalid_type = "invalid_type";

        // Act
        let result = sqlx::query!(
            r#"
            INSERT INTO posts (type, title, slug, content, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
            invalid_type,
            "Test Title",
            "test-slug",
            "Test content",
            "Test description"
        )
        .execute(&db.pool)
        .await;

        // Assert
        assert!(matches!(result, Err(Error::Database(_))));
        Ok(())
    }

    #[sqlx::test]
    async fn test_duplicate_slug_fails() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;
        let slug = "test-slug";

        // Create first post
        sqlx::query!(
            r#"
            INSERT INTO posts (type, title, slug, content, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
            "blog",
            "First Post",
            slug,
            "Content",
            "Description"
        )
        .execute(&db.pool)
        .await?;

        // Act - try to create second post with same slug
        let result = sqlx::query!(
            r#"
            INSERT INTO posts (type, title, slug, content, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
            "blog",
            "Second Post",
            slug,
            "Different content",
            "Different description"
        )
        .execute(&db.pool)
        .await;

        // Assert
        assert!(matches!(result, Err(Error::Database(_))));
        Ok(())
    }

    #[sqlx::test]
    async fn test_post_tag_relationship() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;

        // Create a post
        sqlx::query!(
            r#"
            INSERT INTO posts (type, title, slug, content, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
            "blog",
            "Test Post",
            "test-post",
            "Content",
            "Description"
        )
        .execute(&db.pool)
        .await?;

        // Create a tag
        sqlx::query!("INSERT INTO tags (name) VALUES (?)", "test-tag")
            .execute(&db.pool)
            .await?;

        // Act - Link post and tag
        let result = sqlx::query!(
            r#"
            INSERT INTO post_tags (post_id, tag_id)
            VALUES (
                (SELECT id FROM posts WHERE slug = ?),
                (SELECT id FROM tags WHERE name = ?)
            )
            "#,
            "test-post",
            "test-tag"
        )
        .execute(&db.pool)
        .await?;

        // Assert
        assert_eq!(result.rows_affected(), 1);
        Ok(())
    }

    #[sqlx::test]
    async fn test_cascade_delete_post_removes_tags_relationship() -> sqlx::Result<()> {
        // Arrange
        let db = setup_test_db().await;

        // Create post and tag, then link them
        sqlx::query!(
            r#"
            INSERT INTO posts (type, title, slug, content, description)
            VALUES (?, ?, ?, ?, ?)
            "#,
            "blog",
            "Test Post",
            "test-post",
            "Content",
            "Description"
        )
        .execute(&db.pool)
        .await?;

        sqlx::query!("INSERT INTO tags (name) VALUES (?)", "test-tag")
            .execute(&db.pool)
            .await?;

        sqlx::query!(
            r#"
            INSERT INTO post_tags (post_id, tag_id)
            VALUES (
                (SELECT id FROM posts WHERE slug = ?),
                (SELECT id FROM tags WHERE name = ?)
            )
            "#,
            "test-post",
            "test-tag"
        )
        .execute(&db.pool)
        .await?;

        // Act - Delete the post
        sqlx::query!("DELETE FROM posts WHERE slug = ?", "test-post")
            .execute(&db.pool)
            .await?;

        // Assert - Check that post_tags entry was cascaded
        let remaining_relationships = sqlx::query!("SELECT COUNT(*) as count FROM post_tags")
            .fetch_one(&db.pool)
            .await?;

        assert_eq!(remaining_relationships.count, 0);
        Ok(())
    }
}
