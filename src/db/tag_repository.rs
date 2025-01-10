use sqlx::SqlitePool;
use crate::models::tag::{Tag, TagWithPostCount};

use super::{error::DatabaseResult, DatabaseError};

/// Repository for managing tags in the database
/// Provides methods for creating, reading, updating, and deleting tags,
/// as well as managing relationships between tags and posts
#[derive(Clone, Debug)]
pub struct TagRepository {
    pool: SqlitePool,
}

impl TagRepository {
    /// Creates a new TagRepository instance
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new tag with the given name
    /// Returns an error if a tag with the same name already exists
    pub async fn create(&self, name: &str) -> DatabaseResult<Tag> {
        // Validate tag name
        if name.trim().is_empty() {
            return Err(DatabaseError::validation("Tag name cannot be empty"));
        }

        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        let trimmed_name = name.trim();

        // Attempt to create the tag
        let tag = sqlx::query_as!(
            Tag,
            r#"
            INSERT INTO tags (name)
            VALUES (?)
            RETURNING *
            "#,
            trimmed_name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint") => {
                DatabaseError::duplicate("Tag", name)
            }
            e => DatabaseError::Sqlx(e),
        })?;

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(tag)
    }

    /// Retrieves a tag by its ID
    pub async fn find_by_id(&self, id: i64) -> DatabaseResult<Tag> {
        sqlx::query_as!(
            Tag,
            r#"
            SELECT *
            FROM tags
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::Sqlx)?
        .ok_or_else(|| DatabaseError::not_found("Tag", &id.to_string()))
    }

    /// Retrieves a tag by its name
    pub async fn find_by_name(&self, name: &str) -> DatabaseResult<Tag> {
    sqlx::query_as!(
        Tag,
        r#"
        SELECT 
            id as "id!",
            name as "name!",
            created_at as "created_at!"
        FROM tags
        WHERE name = ?
        "#,
        name
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(DatabaseError::Sqlx)?
    .ok_or_else(|| DatabaseError::not_found("Tag", name))
}

    /// Lists all tags, optionally including the count of posts for each tag
    pub async fn list(&self, include_post_count: bool) -> DatabaseResult<Vec<TagWithPostCount>> {
        let query = if include_post_count {
            r#"
            SELECT 
                t.*,
                COUNT(pt.post_id) as post_count
            FROM tags t
            LEFT JOIN post_tags pt ON t.id = pt.tag_id
            GROUP BY t.id
            ORDER BY t.name
            "#
        } else {
            r#"
            SELECT 
                t.*,
                0 as post_count
            FROM tags t
            ORDER BY t.name
            "#
        };

        sqlx::query_as(query)
            .fetch_all(&self.pool)
            .await
            .map_err(DatabaseError::Sqlx)
    }

    /// Updates a tag's name
    pub async fn update(&self, id: i64, new_name: &str) -> DatabaseResult<Tag> {
        // Validate tag name
        if new_name.trim().is_empty() {
            return Err(DatabaseError::validation("Tag name cannot be empty"));
        }

        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        let trimmed_new_name = new_name.trim();

        let updated_tag = sqlx::query_as!(
            Tag,
            r#"
            UPDATE tags
            SET name = ?
            WHERE id = ?
            RETURNING *
            "#,
            trimmed_new_name,
            id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint") => {
                DatabaseError::duplicate("Tag", new_name)
            }
            e => DatabaseError::Sqlx(e),
        })?
        .ok_or_else(|| DatabaseError::not_found("Tag", &id.to_string()))?;

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(updated_tag)
    }

    /// Deletes a tag by ID
    /// This will also remove all associations between this tag and any posts
    /// due to the ON DELETE CASCADE constraint
    pub async fn delete(&self, id: i64) -> DatabaseResult<()> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        let result = sqlx::query!(
            r#"
            DELETE FROM tags
            WHERE id = ?
            "#,
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::Sqlx)?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("Tag", &id.to_string()));
        }

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(())
    }

    /// Associates a tag with a post
    pub async fn add_tag_to_post(&self, post_id: i64, tag_id: i64) -> DatabaseResult<()> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        sqlx::query!(
            r#"
            INSERT INTO post_tags (post_id, tag_id)
            VALUES (?, ?)
            "#,
            post_id,
            tag_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.message().contains("FOREIGN KEY constraint") => {
                DatabaseError::not_found("Post or Tag", &format!("{post_id}, {tag_id}"))
            }
            sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint") => {
                DatabaseError::duplicate("Tag association", &format!("{post_id}, {tag_id}"))
            }
            e => DatabaseError::Sqlx(e),
        })?;

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(())
    }

    /// Removes a tag association from a post
    pub async fn remove_tag_from_post(&self, post_id: i64, tag_id: i64) -> DatabaseResult<()> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        let result = sqlx::query!(
            r#"
            DELETE FROM post_tags
            WHERE post_id = ? AND tag_id = ?
            "#,
            post_id,
            tag_id
        )
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::Sqlx)?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found(
                "Tag association",
                &format!("{post_id}, {tag_id}")
            ));
        }

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(())
    }

    /// Lists all tags for a specific post
    pub async fn list_tags_for_post(&self, post_id: i64) -> DatabaseResult<Vec<Tag>> {
    sqlx::query_as!(
        Tag,
        r#"
        SELECT 
            t.id as "id!",
            t.name as "name!",
            t.created_at as "created_at!"
        FROM tags t
        JOIN post_tags pt ON t.id = pt.tag_id
        WHERE pt.post_id = ?
        ORDER BY t.name
        "#,
        post_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(DatabaseError::Sqlx)
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ db::{test_utils::create_test_db, Database}, models::post::{CreatePost, PostCategory}};

    async fn setup() -> (Database, TagRepository) {
        let db = create_test_db().await.unwrap();
        let repo = db.tags().clone();
        (db, repo)
    }

    #[tokio::test]
    async fn test_create_tag() {
        let (_, repo) = setup().await;
        
        // Test successful creation
        let tag = repo.create("rust").await;
        assert!(tag.is_ok());
        let tag = tag.unwrap();
        assert_eq!(tag.name, "rust");

        // Test duplicate tag
        let duplicate = repo.create("rust").await;
        assert!(matches!(duplicate.unwrap_err(), DatabaseError::DuplicateEntry(_)));

        // Test empty tag name
        let empty = repo.create("").await;
        assert!(matches!(empty.unwrap_err(), DatabaseError::Validation(_)));

        // Test whitespace handling
        let trimmed = repo.create("  python  ").await.unwrap();
        assert_eq!(trimmed.name, "python");
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let (_, repo) = setup().await;
        
        // Create a test tag
        let created = repo.create("test-tag").await.unwrap();
        
        // Test successful retrieval
        let found = repo.find_by_id(created.id).await;
        assert!(found.is_ok());
        assert_eq!(found.unwrap().name, "test-tag");

        // Test non-existent ID
        let not_found = repo.find_by_id(999).await;
        assert!(matches!(not_found.unwrap_err(), DatabaseError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_find_by_name() {
        let (_, repo) = setup().await;
        
        // Create a test tag
        repo.create("findme").await.unwrap();
        
        // Test successful retrieval
        let found = repo.find_by_name("findme").await;
        assert!(found.is_ok());
        
        // Test non-existent name
        let not_found = repo.find_by_name("nonexistent").await;
        assert!(matches!(not_found.unwrap_err(), DatabaseError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_list_tags() {
        let (_, repo) = setup().await;
        
        // Create some test tags
        repo.create("tag1").await.unwrap();
        repo.create("tag2").await.unwrap();
        
        // Test listing without post count
        let tags = repo.list(false).await.unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].post_count, 0);

        // Test listing with post count
        let tags_with_count = repo.list(true).await.unwrap();
        assert_eq!(tags_with_count.len(), 2);
    }

    #[tokio::test]
    async fn test_update_tag() {
        let (_, repo) = setup().await;
        
        // Create initial tag
        let tag = repo.create("initial").await.unwrap();
        
        // Test successful update
        let updated = repo.update(tag.id, "updated").await;
        assert!(updated.is_ok());
        assert_eq!(updated.unwrap().name, "updated");

        // Test non-existent ID
        let not_found = repo.update(999, "test").await;
        assert!(matches!(not_found.unwrap_err(), DatabaseError::NotFound(_)));

        // Test duplicate name
        repo.create("existing").await.unwrap();
        let duplicate = repo.update(tag.id, "existing").await;
        assert!(matches!(duplicate.unwrap_err(), DatabaseError::DuplicateEntry(_)));
    }

    #[tokio::test]
    async fn test_delete_tag() {
        let (_, repo) = setup().await;
        
        // Create a tag to delete
        let tag = repo.create("delete-me").await.unwrap();
        
        // Test successful deletion
        assert!(repo.delete(tag.id).await.is_ok());
        
        // Verify tag is gone
        assert!(matches!(
            repo.find_by_id(tag.id).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));

        // Test deleting non-existent tag
        assert!(matches!(
            repo.delete(999).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_tag_post_associations() {
        let (db, repo) = setup().await;
        
        // Create test data
        let tag = repo.create("test-tag").await.unwrap();
        let post = db.posts().create(CreatePost{
            category: PostCategory::Blog,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            content: "Test content".to_string(),
            description: "Test description".to_string(),
            image_url: None,
            external_url: None,
            published: true,
        }).await.unwrap();

        // Test adding tag to post
        assert!(repo.add_tag_to_post(post.id, tag.id).await.is_ok());

        // Test duplicate association
        assert!(matches!(
            repo.add_tag_to_post(post.id, tag.id).await.unwrap_err(),
            DatabaseError::DuplicateEntry(_)
        ));

        // Test listing tags for post
        let post_tags = repo.list_tags_for_post(post.id).await.unwrap();
        assert_eq!(post_tags.len(), 1);
        assert_eq!(post_tags[0].id, tag.id);

        // Test removing tag from post
        assert!(repo.remove_tag_from_post(post.id, tag.id).await.is_ok());

        // Verify tag is removed
        let post_tags = repo.list_tags_for_post(post.id).await.unwrap();
        assert!(post_tags.is_empty());

        // Test removing non-existent association
        assert!(matches!(
            repo.remove_tag_from_post(post.id, 999).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));
    }
}
