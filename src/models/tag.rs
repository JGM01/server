use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

/// Represents a tag in the database
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: OffsetDateTime,
}

/// Extended tag information including the count of associated posts
/// Used when listing tags with usage statistics
#[derive(Debug, FromRow, Serialize)]
pub struct TagWithPostCount {
    pub id: i64,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub post_count: i64,
}

impl Tag {
    /// Validates a tag name
    /// Returns true if the name is valid, false otherwise
    pub fn is_valid_name(name: &str) -> bool {
        let trimmed = name.trim();
        !trimmed.is_empty() && trimmed.len() <= 50 && trimmed.chars().all(|c| {
            c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_' || c == '+'
        })
    }
}


#[cfg(test)]
mod tag_tests {
    use super::*;

    #[test]
    fn test_tag_name_validation() {
        // Valid tag names
        assert!(Tag::is_valid_name("rust"));
        assert!(Tag::is_valid_name("web-development"));
        assert!(Tag::is_valid_name("C++"));
        assert!(Tag::is_valid_name("tag with spaces"));
        assert!(Tag::is_valid_name("tag_with_underscore"));

        // Invalid tag names
        assert!(!Tag::is_valid_name("")); // Empty
        assert!(!Tag::is_valid_name("   ")); // Only whitespace
        assert!(!Tag::is_valid_name("tag!")); // Invalid character
        assert!(!Tag::is_valid_name(&"a".repeat(51))); // Too long
    }
}


