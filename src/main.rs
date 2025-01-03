use axum::{http::{header::CONTENT_TYPE, HeaderValue, Method}, routing::get, Json, Router};
use db::Database;
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

pub mod db;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    
    let db = Database::new().await.expect("Failed to initialize database.");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(vec![CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .with_state(db.pool)
        .layer(cors);

    let addr = "127.0.0.1:8080";
    tracing::info!("Serving on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct HealthCheck {
    status: String,
}

async fn health_check() -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "OK".to_string(),
    })
}
