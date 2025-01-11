use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db::Database,
    handlers::{
        post_handlers::{
            create_post, delete_post, get_post_by_id, get_post_by_slug, list_posts, patch_post,
            update_post,
        },
        tag_handlers::{
            add_tag_to_post, create_tag, delete_tag, get_post_tags, get_tag_by_id, get_tag_by_name,
            list_tags, remove_tag_from_post, update_tag,
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
        .route(
            "/posts/{post_id}/tags/{tag_id}",
            delete(remove_tag_from_post),
        )
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
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        response::Response,
    };
    use serde_json::json;
    use tower::ServiceExt;
    // Helper function to create a test app with a database connection
    async fn create_test_app() -> Router {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let db = Database::new().await.unwrap();

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/posts", get(list_posts))
            .route("/posts", post(create_post))
            .route("/posts/by-id/{id}", get(get_post_by_id))
            .route("/posts/by-slug/{slug}", get(get_post_by_slug))
            .route("/posts", put(update_post))
            .route("/posts", patch(patch_post))
            .route("/posts/{id}", delete(delete_post))
            .route("/tags", get(list_tags))
            .route("/tags", post(create_tag))
            .route("/tags/{id}", get(get_tag_by_id))
            .route("/tags/by-name/{name}", get(get_tag_by_name))
            .route("/tags/{id}", put(update_tag))
            .route("/tags/{id}", delete(delete_tag))
            .route("/posts/{post_id}/tags", get(get_post_tags))
            .route("/posts/{post_id}/tags/{tag_id}", put(add_tag_to_post))
            .route(
                "/posts/{post_id}/tags/{tag_id}",
                delete(remove_tag_from_post),
            )
            .with_state(db)
            .layer(cors)
    }

    // Helper to get response body as a Value
    async fn response_json(response: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_cors_configuration() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/posts")
                    .header("Origin", "http://example.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_some());
    }

    #[tokio::test]
    async fn test_post_crud_operations() {
        let app = create_test_app().await;

        // Create post
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/posts")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "category": "blog",
                            "title": "Test Post",
                            "slug": "test-post",
                            "content": "Test content",
                            "description": "Test description",
                            "published": true
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);
        let post = response_json(create_response).await;
        let post_id = post["id"].as_i64().unwrap();

        // Read post
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&format!("/posts/by-id/{}", post_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        // Update post
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PATCH)
                    .uri("/posts")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "id": post_id,
                            "title": "Updated Title"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Delete post
        let delete_response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(&format!("/posts/{}", post_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_tag_operations() {
        let app = create_test_app().await;

        // Create tag
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/tags")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "name": "test-tag"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::OK);
        let tag = response_json(create_response).await;
        let tag_id = tag["id"].as_i64().unwrap();

        // List tags
        let list_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/tags")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        let tags = response_json(list_response).await;
        assert!(tags.as_array().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn test_post_tag_relationships() {
        let app = create_test_app().await;

        // Create post and tag
        let post = response_json(
            app.clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/posts")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            serde_json::to_string(&json!({
                                "category": "blog",
                                "title": "Test Post",
                                "slug": "test-post",
                                "content": "Test content",
                                "description": "Test description",
                                "published": true
                            }))
                            .unwrap(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap(),
        )
        .await;
        let post_id = post["id"].as_i64().unwrap();

        let tag = response_json(
            app.clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/tags")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            serde_json::to_string(&json!({
                                "name": "test-tag"
                            }))
                            .unwrap(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap(),
        )
        .await;
        let tag_id = tag["id"].as_i64().unwrap();

        // Add tag to post
        let add_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri(&format!("/posts/{}/tags/{}", post_id, tag_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(add_response.status(), StatusCode::NO_CONTENT);

        // Get post tags
        let tags_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&format!("/posts/{}/tags", post_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(tags_response.status(), StatusCode::OK);
        let tags = response_json(tags_response).await;
        assert_eq!(tags.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_error_handling() {
        let app = create_test_app().await;

        // Test 404
        let not_found = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);

        // Test method not allowed
        let method_not_allowed = app
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/posts/by-slug/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(method_not_allowed.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
