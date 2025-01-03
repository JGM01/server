use serde::Serialize;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow, Serialize)]
pub struct Post {
    pub id: i64,                            // primary key
    pub type_: String,                      // 'blog', 'art', 'reading'
    pub title: String,                      // headline title of the post
    pub slug: String,                       // url-safe version of title
    pub content: String,                    // markdown-formatted text
    pub description: String,                // bonus snippet that will be a subtle font
    pub image_url: Option<String>,          // only for art posts, big thumbnail image
    pub external_url: Option<String>,       // only for reading, meant to store the linked site/pdf
    pub published: bool,                    // is it published
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

