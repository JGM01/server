use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

use super::errors::PostError;

/// Represents the different categories a post can belong to
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PostCategory {
    Blog,
    Art,
    Reading,
}

// This lets us convert strings into PostCategory values
impl FromStr for PostCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blog" => Ok(PostCategory::Blog),
            "art" => Ok(PostCategory::Art),
            "reading" => Ok(PostCategory::Reading),
            _ => Err(format!("Invalid post category: {}", s)),
        }
    }
}

// This lets us convert PostCategory values into strings
impl ToString for PostCategory {
    fn to_string(&self) -> String {
        match self {
            PostCategory::Blog => "blog".to_string(),
            PostCategory::Art => "art".to_string(),
            PostCategory::Reading => "reading".to_string(),
        }
    }
}

// This handles converting from String to PostCategory for database operations
impl TryFrom<String> for PostCategory {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

// This lets SQLx convert from PostCategory to String for database storage
impl From<PostCategory> for String {
    fn from(category: PostCategory) -> String {
        category.to_string()
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct Post {
    pub id: i64,
    pub category: PostCategory,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub description: String,
    pub image_url: Option<String>,
    pub external_url: Option<String>,
    pub published: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePost {
    pub category: PostCategory,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub description: String,
    pub image_url: Option<String>,
    pub external_url: Option<String>,
    pub published: bool,
}

impl CreatePost {
    pub fn validate(&self) -> Result<(), PostError> {
        if self.title.trim().is_empty() {
            return Err(PostError::EmptyTitle);
        }
        if self.content.trim().is_empty() {
            return Err(PostError::EmptyContent);
        }
        if !is_valid_slug(&self.slug) {
            return Err(PostError::InvalidSlug);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePost {
    pub id: i64,
    pub category: PostCategory,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub description: String,
    pub image_url: Option<String>,
    pub external_url: Option<String>,
    pub published: bool,
}

impl UpdatePost {
    pub fn validate(&self) -> Result<(), PostError> {
        if self.id <= 0 {
            return Err(PostError::InvalidId);
        }
        if self.title.trim().is_empty() {
            return Err(PostError::EmptyTitle);
        }
        if self.content.trim().is_empty() {
            return Err(PostError::EmptyContent);
        }
        if !is_valid_slug(&self.slug) {
            return Err(PostError::InvalidSlug);
        }
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct PatchPost {
    pub id: i64,
    pub category: Option<PostCategory>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub external_url: Option<String>,
    pub published: Option<bool>,
}

fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|c| (c.is_ascii_alphanumeric() || c == '-'))
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}
#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    // Helper function to create a valid CreatePost instance
    fn create_valid_post() -> CreatePost {
        CreatePost {
            category: PostCategory::Blog,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            content: "Test content".to_string(),
            description: "Test description".to_string(),
            image_url: None,
            external_url: None,
            published: false,
        }
    }

    #[test]
    fn test_post_category_conversion() {
        // Test string to PostCategory conversion
        assert_eq!(PostCategory::from_str("blog").unwrap(), PostCategory::Blog);
        assert_eq!(PostCategory::from_str("art").unwrap(), PostCategory::Art);
        assert_eq!(
            PostCategory::from_str("reading").unwrap(),
            PostCategory::Reading
        );

        // Test case insensitivity
        assert_eq!(PostCategory::from_str("BLOG").unwrap(), PostCategory::Blog);
        assert_eq!(PostCategory::from_str("Art").unwrap(), PostCategory::Art);

        // Test invalid category
        assert!(PostCategory::from_str("invalid").is_err());
    }

    #[test]
    fn test_post_category_to_string() {
        assert_eq!(PostCategory::Blog.to_string(), "blog");
        assert_eq!(PostCategory::Art.to_string(), "art");
        assert_eq!(PostCategory::Reading.to_string(), "reading");
    }

    #[test]
    fn test_create_post_validation() {
        // Test valid post
        let valid_post = create_valid_post();
        assert!(valid_post.validate().is_ok());

        // Test empty title
        let mut invalid_post = create_valid_post();
        invalid_post.title = "".to_string();
        assert!(matches!(
            invalid_post.validate(),
            Err(PostError::EmptyTitle)
        ));

        // Test whitespace title
        let mut whitespace_post = create_valid_post();
        whitespace_post.title = "    ".to_string();
        assert!(matches!(
            whitespace_post.validate(),
            Err(PostError::EmptyTitle)
        ));

        // Test empty content
        let mut no_content_post = create_valid_post();
        no_content_post.content = "".to_string();
        assert!(matches!(
            no_content_post.validate(),
            Err(PostError::EmptyContent)
        ));

        // Test invalid slug
        let mut invalid_slug_post = create_valid_post();
        invalid_slug_post.slug = "invalid slug!".to_string();
        assert!(matches!(
            invalid_slug_post.validate(),
            Err(PostError::InvalidSlug)
        ));
    }

    #[test]
    fn test_update_post_validation() {
        // Test valid update
        let valid_update = UpdatePost {
            id: 1,
            category: PostCategory::Blog,
            title: "Updated Post".to_string(),
            slug: "updated-post".to_string(),
            content: "Updated content".to_string(),
            description: "Updated description".to_string(),
            image_url: None,
            external_url: None,
            published: true,
        };
        assert!(valid_update.validate().is_ok());

        // Test invalid ID
        let mut invalid_id = valid_update.clone();
        invalid_id.id = 0;
        assert!(matches!(invalid_id.validate(), Err(PostError::InvalidId)));

        invalid_id.id = -1;
        assert!(matches!(invalid_id.validate(), Err(PostError::InvalidId)));

        // Test empty fields
        let mut empty_fields = valid_update.clone();
        empty_fields.title = "".to_string();
        assert!(matches!(
            empty_fields.validate(),
            Err(PostError::EmptyTitle)
        ));

        empty_fields = valid_update.clone();
        empty_fields.content = "".to_string();
        assert!(matches!(
            empty_fields.validate(),
            Err(PostError::EmptyContent)
        ));
    }

    #[test]
    fn test_slug_validation() {
        // Test valid slugs
        assert!(is_valid_slug("simple-slug"));
        assert!(is_valid_slug("123-numeric"));
        assert!(is_valid_slug("multiple-hyphens-are-ok"));
        assert!(is_valid_slug("alphanumeric123"));

        // Test invalid slugs
        assert!(!is_valid_slug("")); // Empty
        assert!(!is_valid_slug("-starts-with-hyphen")); // Starting hyphen
        assert!(!is_valid_slug("ends-with-hyphen-")); // Ending hyphen
        assert!(!is_valid_slug("special!chars")); // Special characters
        assert!(!is_valid_slug("spaces not allowed")); // Spaces
    }

    #[test]
    fn test_patch_post_default() {
        // Test Default implementation for PatchPost
        let patch = PatchPost::default();
        assert_eq!(patch.id, 0);
        assert!(patch.category.is_none());
        assert!(patch.title.is_none());
        assert!(patch.slug.is_none());
        assert!(patch.content.is_none());
        assert!(patch.description.is_none());
        assert!(patch.image_url.is_none());
        assert!(patch.external_url.is_none());
        assert!(patch.published.is_none());
    }

    #[test]
    fn test_post_urls() {
        // Test URL validation (if implemented)
        let mut post = create_valid_post();

        // Valid URLs
        post.image_url = Some("https://example.com/image.jpg".to_string());
        post.external_url = Some("https://example.com/article".to_string());
        assert!(post.validate().is_ok());

        // Invalid URLs (if we implement URL validation)
        post.image_url = Some("not-a-url".to_string());
        // Assuming we add URL validation, this would fail:
        // assert!(post.validate().is_err());
    }

    #[test]
    fn test_post_timestamps() {
        let now = OffsetDateTime::now_utc();
        let post = Post {
            id: 1,
            category: PostCategory::Blog,
            title: "Test".to_string(),
            slug: "test".to_string(),
            content: "Content".to_string(),
            description: "Description".to_string(),
            image_url: None,
            external_url: None,
            published: false,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(post.created_at, now);
        assert_eq!(post.updated_at, now);
    }
}
