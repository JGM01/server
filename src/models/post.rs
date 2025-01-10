

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
        && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}

#[cfg(test)]
mod post_tests {
    use super::*;

    #[test]
    fn test_post_category_from_str() {
        assert_eq!(PostCategory::from_str("blog").unwrap(), PostCategory::Blog);
        assert_eq!(PostCategory::from_str("art").unwrap(), PostCategory::Art);
        assert_eq!(PostCategory::from_str("reading").unwrap(), PostCategory::Reading);
        
        // Case insensitive
        assert_eq!(PostCategory::from_str("BLOG").unwrap(), PostCategory::Blog);
        assert_eq!(PostCategory::from_str("Art").unwrap(), PostCategory::Art);
        
        // Invalid categories
        assert!(PostCategory::from_str("invalid").is_err());
        assert!(PostCategory::from_str("").is_err());
    }

    #[test]
    fn test_post_category_to_string() {
        assert_eq!(PostCategory::Blog.to_string(), "blog");
        assert_eq!(PostCategory::Art.to_string(), "art");
        assert_eq!(PostCategory::Reading.to_string(), "reading");
    }


    #[test]
    fn test_slug_validation() {
        // Valid slugs
        assert!(is_valid_slug("valid-slug"));
        assert!(is_valid_slug("123"));
        assert!(is_valid_slug("post-123"));
        assert!(is_valid_slug("valid-slug-123"));

        // Invalid slugs
        assert!(!is_valid_slug("")); // Empty
        assert!(!is_valid_slug("-invalid")); // Starts with hyphen
        assert!(!is_valid_slug("invalid-")); // Ends with hyphen
        assert!(!is_valid_slug("invalid!")); // Invalid characters
        assert!(!is_valid_slug("invalid space")); // Contains space
    }
}
