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
        !trimmed.is_empty()
            && trimmed.len() <= 50
            && trimmed.chars().all(|c| {
                c.is_ascii_alphanumeric() || c.is_whitespace() || c == '-' || c == '_' || c == '+'
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    #[test]
    fn test_valid_tag_name_basic() {
        // Test basic alphanumeric names
        assert!(
            Tag::is_valid_name("rust"),
            "Simple lowercase name should be valid"
        );
        assert!(
            Tag::is_valid_name("React"),
            "Name with capital letters should be valid"
        );
        assert!(
            Tag::is_valid_name("vue3"),
            "Name with numbers should be valid"
        );
    }

    #[test]
    fn test_valid_tag_name_with_special_chars() {
        // Test allowed special characters
        assert!(
            Tag::is_valid_name("web-development"),
            "Hyphenated name should be valid"
        );
        assert!(
            Tag::is_valid_name("ruby_on_rails"),
            "Underscore should be valid"
        );
        assert!(
            Tag::is_valid_name("C++"),
            "Programming language names should be valid"
        );
        assert!(Tag::is_valid_name("Front End"), "Spaces should be valid");
    }

    #[test]
    fn test_invalid_tag_name_empty() {
        // Test empty and whitespace names
        assert!(!Tag::is_valid_name(""), "Empty string should be invalid");
        assert!(
            !Tag::is_valid_name("   "),
            "Whitespace only should be invalid"
        );
        assert!(
            !Tag::is_valid_name("\t\n"),
            "Special whitespace should be invalid"
        );
    }

    #[test]
    fn test_invalid_tag_name_special_chars() {
        // Test disallowed special characters
        assert!(
            !Tag::is_valid_name("tag!"),
            "Exclamation mark should be invalid"
        );
        assert!(
            !Tag::is_valid_name("tag@web"),
            "At symbol should be invalid"
        );
        assert!(!Tag::is_valid_name("#trending"), "Hash should be invalid");
        assert!(!Tag::is_valid_name("tag$"), "Dollar sign should be invalid");
    }

    #[test]
    fn test_invalid_tag_name_length() {
        // Test maximum length constraint
        let long_name = "a".repeat(51);
        assert!(
            !Tag::is_valid_name(&long_name),
            "Names longer than 50 chars should be invalid"
        );

        let max_length_name = "a".repeat(50);
        assert!(
            Tag::is_valid_name(&max_length_name),
            "50 char name should be valid"
        );
    }

    #[test]
    fn test_tag_with_post_count_creation() {
        // Test creating TagWithPostCount
        let now = OffsetDateTime::now_utc();
        let tag_with_count = TagWithPostCount {
            id: 1,
            name: "test".to_string(),
            created_at: now,
            post_count: 5,
        };

        assert_eq!(tag_with_count.id, 1);
        assert_eq!(tag_with_count.name, "test");
        assert_eq!(tag_with_count.created_at, now);
        assert_eq!(tag_with_count.post_count, 5);
    }

    #[test]
    fn test_tag_name_trimming() {
        // Test that leading/trailing whitespace doesn't affect validation
        assert!(
            Tag::is_valid_name(" rust "),
            "Names with surrounding spaces should be valid"
        );
        assert!(
            Tag::is_valid_name("\tpython\n"),
            "Names with surrounding whitespace should be valid"
        );
    }

    #[test]
    fn test_tag_name_unicode() {
        // Test Unicode character handling
        assert!(
            !Tag::is_valid_name("标签"),
            "Non-ASCII characters should be invalid"
        );
        assert!(
            !Tag::is_valid_name("タグ"),
            "Japanese characters should be invalid"
        );
        assert!(!Tag::is_valid_name("🏷️"), "Emoji should be invalid");
    }
}
