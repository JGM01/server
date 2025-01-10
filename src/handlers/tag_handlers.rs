use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    db::Database,
    models::tag::{Tag, TagWithPostCount},
};

// We'll reuse the ApiError from post_handlers.rs, so let's import it
use super::post_handlers::ApiError;

/// Request body for creating or updating a tag
#[derive(Debug, Deserialize)]
pub struct TagRequest {
    pub name: String,
}

/// Query parameters for listing tags
#[derive(Debug, Deserialize)]
pub struct ListTagsQuery {
    #[serde(default)]
    pub include_post_count: bool,
}

/// Create a new tag
/// 
/// This handler accepts a JSON payload containing the tag name and creates
/// a new tag in the database. It ensures the tag name is unique.
pub async fn create_tag(
    State(db): State<Database>,
    Json(tag_request): Json<TagRequest>,
) -> Result<Json<Tag>, ApiError> {
    // Validate tag name format before attempting database operation
    if !Tag::is_valid_name(&tag_request.name) {
        return Err(ApiError::InvalidInput("Invalid tag name format".to_string()));
    }

    let tag = db.tags().create(&tag_request.name).await?;
    Ok(Json(tag))
}

/// Get a tag by its ID
/// 
/// This handler retrieves a single tag by its database ID. It returns a 404
/// error if the tag is not found.
pub async fn get_tag_by_id(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> Result<Json<Tag>, ApiError> {
    let tag = db.tags().find_by_id(id).await?;
    Ok(Json(tag))
}

/// Get a tag by its name
/// 
/// This handler retrieves a single tag by its name. It returns a 404
/// error if the tag is not found.
pub async fn get_tag_by_name(
    State(db): State<Database>,
    Path(name): Path<String>,
) -> Result<Json<Tag>, ApiError> {
    let tag = db.tags().find_by_name(&name).await?;
    Ok(Json(tag))
}

/// List all tags
/// 
/// This handler returns a list of all tags, optionally including the count
/// of posts associated with each tag.
pub async fn list_tags(
    State(db): State<Database>,
    Query(query): Query<ListTagsQuery>,
) -> Result<Json<Vec<TagWithPostCount>>, ApiError> {
    let tags = db.tags().list(query.include_post_count).await?;
    Ok(Json(tags))
}

/// Update a tag's name
/// 
/// This handler accepts a JSON payload containing the new tag name and updates
/// the tag with the specified ID.
pub async fn update_tag(
    State(db): State<Database>,
    Path(id): Path<i64>,
    Json(tag_request): Json<TagRequest>,
) -> Result<Json<Tag>, ApiError> {
    // Validate tag name format before attempting database operation
    if !Tag::is_valid_name(&tag_request.name) {
        return Err(ApiError::InvalidInput("Invalid tag name format".to_string()));
    }

    let tag = db.tags().update(id, &tag_request.name).await?;
    Ok(Json(tag))
}

/// Delete a tag
/// 
/// This handler deletes the tag with the specified ID. It returns a 404
/// error if the tag is not found. Due to the database's foreign key
/// constraints, this will also remove all associations between this tag
/// and any posts.
pub async fn delete_tag(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    db.tags().delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add a tag to a post
/// 
/// This handler creates an association between a post and a tag. Both the
/// post and tag must exist.
pub async fn add_tag_to_post(
    State(db): State<Database>,
    Path((post_id, tag_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    db.tags().add_tag_to_post(post_id, tag_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove a tag from a post
/// 
/// This handler removes the association between a post and a tag. Returns
/// a 404 error if either the post or tag doesn't exist, or if they're not
/// associated.
pub async fn remove_tag_from_post(
    State(db): State<Database>,
    Path((post_id, tag_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    db.tags().remove_tag_from_post(post_id, tag_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get all tags for a post
/// 
/// This handler returns a list of all tags associated with the specified post.
pub async fn get_post_tags(
    State(db): State<Database>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    let tags = db.tags().list_tags_for_post(post_id).await?;
    Ok(Json(tags))
}

