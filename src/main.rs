use std::net::SocketAddr;
use axum::{
    routing::{get, post, put, patch, delete},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db::Database,
    handlers::{
        post_handlers::{
            create_post, delete_post, get_post_by_id, get_post_by_slug,
            list_posts, patch_post, update_post,
        },
        tag_handlers::{
            add_tag_to_post, create_tag, delete_tag, get_post_tags,
            get_tag_by_id, get_tag_by_name, list_tags, remove_tag_from_post,
            update_tag,
        },
    },
};

mod db;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database connection
    let db = Database::new().await?;

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build routes
    let app = Router::new()
        // Post routes
        .route("/posts", get(list_posts))
        .route("/posts", post(create_post))
        .route("/posts/by-id/{id}", get(get_post_by_id))
        .route("/posts/by-slug/{slug}", get(get_post_by_slug))
        .route("/posts", put(update_post))
        .route("/posts", patch(patch_post))
        .route("/posts/{id}", delete(delete_post))
        // Tag routes
        .route("/tags", get(list_tags))
        .route("/tags", post(create_tag))
        .route("/tags/{id}", get(get_tag_by_id))
        .route("/tags/by-name/{name}", get(get_tag_by_name))
        .route("/tags/{id}", put(update_tag))
        .route("/tags/{id}", delete(delete_tag))
        // Post-Tag relationship routes
        .route("/posts/{post_id}/tags", get(get_post_tags))
        .route("/posts/{post_id}/tags/{tag_id}", put(add_tag_to_post))
        .route("/posts/{post_id}/tags/{tag_id}", delete(remove_tag_from_post))
        // Add database state and middleware
        .with_state(db)
        .layer(cors);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address {}", addr));

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e));

    Ok(())
}
