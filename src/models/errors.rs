use thiserror::Error;

/// Represents all possible errors that can occur when working with posts.
/// Using thiserror to automatically derive Error implementations makes our error
/// handling more maintainable and provides better error messages.
#[derive(Debug, Error)]
pub enum PostError {
    #[error("Post ID must be positive")]
    InvalidId,

    #[error("Post title cannot be empty")]
    EmptyTitle,

    #[error("Post content cannot be empty")]
    EmptyContent,

    #[error("Invalid slug format")]
    InvalidSlug,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
