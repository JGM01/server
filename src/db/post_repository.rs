/// Repository for managing post entities in the database.
/// This struct provides a clean API for all post-related database operations,
/// handling connections, transactions, and error mapping.
#[derive(Clone, Debug)]
pub struct PostRepository {
    pool: SqlitePool,
}

impl PostRepository {
    /// Creates a new PostRepository instance.
    /// The repository takes ownership of a connection pool clone, allowing
    /// multiple repositories to share the same pool.
    pub(crate) fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Creates a new post in the database.
    /// This method handles validation, insertion, and returns the complete
    /// post record with generated fields like ID and timestamps.
    pub async fn create(&self, post: CreatePost) -> DatabaseResult<Post> {
        // Validate all fields before attempting database operation
        post.validate()
            .map_err(|e| DatabaseError::Validation(e.to_string()))?;

        // Start a transaction to ensure data consistency
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        // Convert category to string for database storage
        let category_str = post.category.to_string();

        let created_post = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (
                category,
                title, 
                slug,
                content,
                description,
                image_url,
                external_url,
                published
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, category as "category: PostCategory", title, slug,
                content, description, image_url, external_url, published,
                created_at, updated_at
            "#,
            category_str,
            post.title,
            post.slug,
            post.content,
            post.description,
            post.image_url,
            post.external_url,
            post.published
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint") => {
                DatabaseError::duplicate("Post", &post.slug)
            }
            e => DatabaseError::Sqlx(e),
        })?;

        // Commit the transaction
        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(created_post)
    }

    /// Retrieves a post by its unique identifier.
    /// Returns a NotFound error if the post doesn't exist.
    pub async fn find_by_id(&self, id: i64) -> DatabaseResult<Post> {
        sqlx::query_as!(
            Post,
            r#"
            SELECT 
                id, category as "category: PostCategory", title, slug,
                content, description, image_url, external_url, published,
                created_at, updated_at
            FROM posts
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::Sqlx)?
        .ok_or_else(|| DatabaseError::not_found("Post", &id.to_string()))
    }

    /// Retrieves a post by its URL-friendly slug.
    /// Returns a NotFound error if the post doesn't exist.
    pub async fn find_by_slug(&self, slug: &str) -> DatabaseResult<Post> {
        sqlx::query_as!(
            Post,
            r#"
        SELECT 
            id as "id!", 
            category as "category!: PostCategory", 
            title as "title!", 
            slug as "slug!", 
            content as "content!", 
            description as "description!", 
            image_url, 
            external_url,
            published as "published!",
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM posts
        WHERE slug = ?
        "#,
            slug
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DatabaseError::Sqlx)?
        .ok_or_else(|| DatabaseError::not_found("Post", slug))
    }

    /// Lists posts with optional filtering and pagination.
    ///
    /// Parameters:
    /// - category: Optional filter for post category
    /// - published_only: When true, returns only published posts
    /// - limit: Maximum number of posts to return (1-100)
    /// - offset: Number of posts to skip for pagination
    pub async fn list(
        &self,
        category: Option<PostCategory>,
        published_only: bool,
        limit: i64,
        offset: i64,
    ) -> DatabaseResult<Vec<Post>> {
        // Validate pagination parameters
        if limit <= 0 || limit > 100 {
            return Err(DatabaseError::validation("Limit must be between 1 and 100"));
        }
        if offset < 0 {
            return Err(DatabaseError::validation("Offset cannot be negative"));
        }

        // Convert category to string if it exists
        let category_str = category.map(|c| c.to_string());

        sqlx::query_as!(
            Post,
            r#"
            SELECT 
                id, category as "category: PostCategory", title, slug,
                content, description, image_url, external_url, published,
                created_at, updated_at
            FROM posts
            WHERE
                (? IS NULL OR category = ?)
                AND (? = FALSE OR published = TRUE)
            ORDER BY created_at DESC
            LIMIT ?
            OFFSET ?
            "#,
            category_str,
            category_str,
            published_only,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::Sqlx)
    }

    /// Updates all fields of an existing post.
    /// Returns a NotFound error if the post doesn't exist.
    pub async fn update(&self, post: UpdatePost) -> DatabaseResult<Post> {
        // Validate all fields before attempting database operation
        post.validate()
            .map_err(|e| DatabaseError::Validation(e.to_string()))?;

        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        // Convert category to string for database storage
        let category_str = post.category.to_string();

        let updated_post = sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET
                category = ?,
                title = ?,
                slug = ?,
                content = ?,
                description = ?,
                image_url = ?,
                external_url = ?,
                published = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            RETURNING 
                id, category as "category: PostCategory", title, slug,
                content, description, image_url, external_url, published,
                created_at, updated_at
            "#,
            category_str,
            post.title,
            post.slug,
            post.content,
            post.description,
            post.image_url,
            post.external_url,
            post.published,
            post.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint") => {
                DatabaseError::duplicate("Post", &post.slug)
            }
            e => DatabaseError::Sqlx(e),
        })?
        .ok_or_else(|| DatabaseError::not_found("Post", &post.id.to_string()))?;

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(updated_post)
    }

    /// Partially updates a post, only modifying provided fields.
    /// This is useful for making small changes without needing to send the entire post.
    pub async fn patch(&self, patch: PatchPost) -> DatabaseResult<Post> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        // First fetch the existing post to merge with patch data
        let current = self.find_by_id(patch.id).await?;

        // Convert category to string if it's being updated
        let category_str = patch.category.unwrap_or(current.category).to_string();

        let title = patch.title.clone().unwrap_or(current.title);
        let slug = patch.slug.clone().unwrap_or(current.slug);
        let content = patch.content.unwrap_or(current.content);
        let description = patch.description.unwrap_or(current.description);
        let img = patch.image_url.or(current.image_url);
        let url = patch.external_url.or(current.external_url);
        let published = patch.published.unwrap_or(current.published);
        let updated_post = sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET
                category = ?,
                title = ?,
                slug = ?,
                content = ?,
                description = ?,
                image_url = ?,
                external_url = ?,
                published = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            RETURNING 
                id, category as "category: PostCategory", title, slug,
                content, description, image_url, external_url, published,
                created_at, updated_at
            "#,
            category_str,
            title,
            slug,
            content,
            description,
            img,
            url,
            published,
            patch.id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.message().contains("UNIQUE constraint") => {
                DatabaseError::duplicate("Post", &patch.slug.unwrap_or_default())
            }
            e => DatabaseError::Sqlx(e),
        })?;

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(updated_post)
    }

    /// Deletes a post by its ID.
    /// Returns a NotFound error if the post doesn't exist.
    pub async fn delete(&self, id: i64) -> DatabaseResult<()> {
        let mut tx = self.pool.begin().await.map_err(DatabaseError::Sqlx)?;

        let result = sqlx::query!(
            r#"
            DELETE FROM posts
            WHERE id = ?
            "#,
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(DatabaseError::Sqlx)?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::not_found("Post", &id.to_string()));
        }

        tx.commit().await.map_err(DatabaseError::Sqlx)?;
        Ok(())
    }
}
use sqlx::SqlitePool;

use crate::models::post::{CreatePost, PatchPost, Post, PostCategory, UpdatePost};

use super::{error::DatabaseResult, DatabaseError};

#[cfg(test)]
mod tests {
    use crate::db::{test_utils::create_test_db, Database};

    use super::*;

    fn create_test_post() -> CreatePost {
        CreatePost {
            category: PostCategory::Blog,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            content: "Test content".to_string(),
            description: "Test description".to_string(),
            image_url: None,
            external_url: None,
            published: true,
        }
    }

    async fn setup() -> (Database, PostRepository) {
        let db = create_test_db().await.unwrap();
        let repo = db.posts().clone();
        (db, repo)
    }

    #[tokio::test]
    async fn test_create_post() {
        let (_, repo) = setup().await;
        let post_data = create_test_post();

        // Test successful creation
        let post = repo.create(post_data.clone()).await;
        assert!(post.is_ok());
        let post = post.unwrap();
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.slug, "test-post");

        // Test duplicate slug
        let duplicate = repo.create(post_data).await;
        assert!(matches!(
            duplicate.unwrap_err(),
            DatabaseError::DuplicateEntry(_)
        ));

        // Test empty title
        let mut invalid = create_test_post();
        invalid.title = "".to_string();
        assert!(matches!(
            repo.create(invalid).await.unwrap_err(),
            DatabaseError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let (_, repo) = setup().await;

        // Create test post
        let created = repo.create(create_test_post()).await.unwrap();

        // Test successful retrieval
        let found = repo.find_by_id(created.id).await;
        assert!(found.is_ok());
        assert_eq!(found.unwrap().title, "Test Post");

        // Test non-existent ID
        let not_found = repo.find_by_id(999).await;
        assert!(matches!(not_found.unwrap_err(), DatabaseError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_find_by_slug() {
        let (_, repo) = setup().await;

        // Create test post
        repo.create(create_test_post()).await.unwrap();

        // Test successful retrieval
        let found = repo.find_by_slug("test-post").await;
        assert!(found.is_ok());

        // Test non-existent slug
        let not_found = repo.find_by_slug("nonexistent").await;
        assert!(matches!(not_found.unwrap_err(), DatabaseError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_list_posts() {
        let (_, repo) = setup().await;

        // Create some test posts
        let mut post1 = create_test_post();
        post1.slug = "post-1".to_string();
        let mut post2 = create_test_post();
        post2.slug = "post-2".to_string();
        post2.category = PostCategory::Art;
        post2.published = false;

        repo.create(post1).await.unwrap();
        repo.create(post2).await.unwrap();

        // Test listing all posts
        let all_posts = repo.list(None, false, 10, 0).await.unwrap();
        assert_eq!(all_posts.len(), 2);

        // Test category filter
        let blog_posts = repo
            .list(Some(PostCategory::Blog), false, 10, 0)
            .await
            .unwrap();
        assert_eq!(blog_posts.len(), 1);

        // Test published filter
        let published = repo.list(None, true, 10, 0).await.unwrap();
        assert_eq!(published.len(), 1);

        // Test pagination
        let paginated = repo.list(None, false, 1, 1).await.unwrap();
        assert_eq!(paginated.len(), 1);

        // Test invalid pagination
        assert!(repo.list(None, false, 0, 0).await.is_err());
        assert!(repo.list(None, false, 10, -1).await.is_err());
    }

    #[tokio::test]
    async fn test_update_post() {
        let (_, repo) = setup().await;

        // Create initial post
        let created = repo.create(create_test_post()).await.unwrap();

        // Test successful update
        let update = UpdatePost {
            id: created.id,
            category: PostCategory::Art,
            title: "Updated Title".to_string(),
            slug: "updated-slug".to_string(),
            content: "Updated content".to_string(),
            description: "Updated description".to_string(),
            image_url: Some("https://example.com/image.jpg".to_string()),
            external_url: Some("https://example.com".to_string()),
            published: false,
        };

        let updated = repo.update(update.clone()).await.unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.slug, "updated-slug");
        assert_eq!(updated.category, PostCategory::Art);
        assert!(!updated.published);

        // Test non-existent ID
        let mut invalid = update;
        invalid.id = 999;
        assert!(matches!(
            repo.update(invalid).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_patch_post() {
        let (_, repo) = setup().await;

        // Create initial post
        let created = repo.create(create_test_post()).await.unwrap();

        // Test partial update with only title
        let patch = PatchPost {
            id: created.id,
            title: Some("Patched Title".to_string()),
            category: None,
            slug: None,
            content: None,
            description: None,
            image_url: None,
            external_url: None,
            published: None,
        };

        let patched = repo.patch(patch).await.unwrap();
        assert_eq!(patched.title, "Patched Title");
        // Other fields should remain unchanged
        assert_eq!(patched.slug, "test-post");
        assert_eq!(patched.category, PostCategory::Blog);
        assert!(patched.published);

        // Test patch with multiple fields
        let multi_patch = PatchPost {
            id: created.id,
            category: Some(PostCategory::Art),
            published: Some(false),
            ..Default::default()
        };

        let multi_patched = repo.patch(multi_patch).await.unwrap();
        assert_eq!(multi_patched.category, PostCategory::Art);
        assert!(!multi_patched.published);
        // Unpatched fields should remain unchanged
        assert_eq!(multi_patched.title, "Patched Title");

        // Test non-existent ID
        let invalid_patch = PatchPost {
            id: 999,
            title: Some("Invalid".to_string()),
            ..Default::default()
        };
        assert!(matches!(
            repo.patch(invalid_patch).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_delete_post() {
        let (_, repo) = setup().await;

        // Create a post to delete
        let post = repo.create(create_test_post()).await.unwrap();

        // Test successful deletion
        assert!(repo.delete(post.id).await.is_ok());

        // Verify post is gone
        assert!(matches!(
            repo.find_by_id(post.id).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));

        // Test deleting non-existent post
        assert!(matches!(
            repo.delete(999).await.unwrap_err(),
            DatabaseError::NotFound(_)
        ));
    }
}
