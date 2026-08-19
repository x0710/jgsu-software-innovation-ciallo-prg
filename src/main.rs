mod db;
mod models;
mod routes;

use axum::{routing::{get, post}, Router};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "soft26app=info".into()),
        )
        .init();

    let pool = db::create_pool().await.expect("Failed to create pool");

    db::seed_data(&pool)
        .await
        .expect("Failed to seed data");

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/api/questions", post(routes::create_question))
        .route("/api/questions", get(routes::questions_api))
        .route("/api/questions/answer", post(routes::answer_question))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = std::env::var("ADDR").unwrap_or_else(|_| "[::]:3000".to_string());

    tracing::info!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Failed to serve");
}

#[allow(dead_code)]
fn _use_pool(_pool: &SqlitePool) {}
