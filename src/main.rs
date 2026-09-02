mod db;
mod models;
mod routes;

use std::io::Write;
use std::path::Path;
use axum::{routing::{get, post}, Router};
use sqlx::SqlitePool;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{debug, info, trace, warn};

pub const QUESTION: &'static str = "/api/questions";
pub const ANSWER: &'static str = "/api/quesitons/answer";

#[tokio::main]
async fn main() {
    if dotenv::dotenv().is_err() {
        if let Ok(mut file) = std::fs::File::create("./.env") {
            file.write_all(
                br#"DATABASE_URL=sqlite://./blog.db
ADDR=[::]:3000
RUST_LOG=info
"#,
            )
                .inspect(|_| info!("Created .env file"))
                .inspect_err(|e| warn!(err=?e, "Failed to create .env file"))
                .ok();
        }

    }

    let log_level = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "soft26app=info".into());
    info!("Logging initialized: {}", log_level);

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./blog.db".to_string());

    // SQLite 数据库文件路径
    let db_path = database_url
        .strip_prefix("sqlite://")
        .unwrap_or(&database_url);
    info!("Using database at: {}", db_path);

    // 如果父目录存在，则创建父目录
    let db_path = Path::new(db_path);
    if let Some(parent) = db_path.parent() {
        debug!("Database parent directory: {:?}", parent.canonicalize());

        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .expect("Failed to create parent directory");
        }
        if !db_path.exists() {
            std::fs::File::create(&db_path).expect("Failed to create db file");
        }
    }

    let pool = SqlitePool::connect(&database_url).await
        .expect("Failed to connect to the database");

    let pool = db::create_pool(pool)
        .await
        .expect("Failed to create pool");

    db::seed_data(&pool)
        .await
        .expect("Failed to seed data");

    let app = Router::new()
        .route("/", get(routes::index))
        .route(QUESTION, post(routes::create_question))
        .route(QUESTION, get(routes::questions_api))
        .route(ANSWER, post(routes::answer_question))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(pool);
    trace!("API list: {:?}", app);
    info!("Question api: {}", QUESTION);
    info!("Answer api: {}", ANSWER);

    let addr = std::env::var("ADDR").unwrap_or_else(|_| "[::]:3000".to_string());

    info!("Server will be running at `http://{}`", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .await
        .expect("Failed to serve");

}
