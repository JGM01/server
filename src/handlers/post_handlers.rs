use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    db::{Database, DatabaseError},
    models::post::{CreatePost, PatchPost, Post, PostCategory, UpdatePost},
};

/// Query parameters for listing posts with pagination and filtering options
#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub category: Option<String>,
    #[serde(default)]
    pub published_only: bool,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Default number of posts to return in a single request
fn default_limit() -> i64 {
    20
}

/// Custom error type for our API endpoints that maps both database
/// and validation errors to appropriate HTTP responses
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Convert our ApiError into appropriate HTTP responses
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::Database(DatabaseError::NotFound(msg)) => (StatusCode::NOT_FOUND, msg),
            ApiError::Database(DatabaseError::DuplicateEntry(msg)) => (StatusCode::CONFLICT, msg),
            ApiError::Database(DatabaseError::Validation(msg)) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(ErrorResponse { message });
        (status, body).into_response()
    }
}

/// Consistent error response structure for all API errors
#[derive(serde::Serialize)]
struct ErrorResponse {
    message: String,
}

/// Create a new post
/// 
/// This handler validates the input and creates a new post in the database.
/// Returns the created post with its ID and timestamps on success.
pub async fn create_post(
    State(db): State<Database>,
    Json(create_post): Json<CreatePost>,
) -> Result<Json<Post>, ApiError> {
    let post = db.posts().create(create_post).await?;
    Ok(Json(post))
}

/// Retrieve a post by its database ID
pub async fn get_post_by_id(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> Result<Json<Post>, ApiError> {
    let post = db.posts().find_by_id(id).await?;
    Ok(Json(post))
}

/// Retrieve a post by its URL-friendly slug
pub async fn get_post_by_slug(
    State(db): State<Database>,
    Path(slug): Path<String>,
) -> Result<Json<Post>, ApiError> {
    let post = db.posts().find_by_slug(&slug).await?;
    Ok(Json(post))
}

/// List posts with optional filtering and pagination
/// 
/// Supports filtering by:
/// - Category (blog, art, reading)
/// - Publication status (draft/published)
/// 
/// And pagination using:
/// - limit (max number of posts to return)
/// - offset (number of posts to skip)
pub async fn list_posts(
    State(db): State<Database>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<Vec<Post>>, ApiError> {
    let category = match query.category {
        Some(cat_str) => Some(PostCategory::from_str(&cat_str)
            .map_err(|e| ApiError::InvalidInput(format!("Invalid category: {}", e)))?),
        None => None,
    };

    let posts = db
        .posts()
        .list(
            category,
            query.published_only,
            query.limit,
            query.offset,
        )
        .await?;
    Ok(Json(posts))
}

/// Update all fields of an existing post
/// 
/// This is a full update that requires all fields to be provided.
/// For partial updates, use the patch_post handler instead.
pub async fn update_post(
    State(db): State<Database>,
    Json(update_post): Json<UpdatePost>,
) -> Result<Json<Post>, ApiError> {
    let post = db.posts().update(update_post).await?;
    Ok(Json(post))
}

/// Partially update a post
/// 
/// Allows updating only specific fields of a post while leaving others unchanged.
/// This is useful for small updates like toggling publication status or updating
/// the title without having to provide all other fields.
pub async fn patch_post(
    State(db): State<Database>,
    Json(patch_post): Json<PatchPost>,
) -> Result<Json<Post>, ApiError> {
    let post = db.posts().patch(patch_post).await?;
    Ok(Json(post))
}

/// Delete a post by its ID
/// 
/// If the post has any tags, the associations will be automatically removed
/// thanks to the ON DELETE CASCADE constraint in our database schema.
pub async fn delete_post(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    db.posts().delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

