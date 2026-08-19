use crate::models::{NewAnswer, NewQuestion, Question, TimelineEvent};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
};
use sqlx::SqlitePool;
use tracing::error;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    questions: Vec<Question>,
    timeline: Vec<TimelineEvent>,
}

pub async fn index(State(pool): State<SqlitePool>) -> Result<Html<String>, StatusCode> {
    let questions = get_questions(&pool).await
        .map_err(|e| {
            error!(error=?e, "error getting questions");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let mut timeline = get_timeline(&pool).await
        .map_err(|e| {
            error!(error=?e, "error getting timeline");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    timeline.sort();

    let template = IndexTemplate {
        questions,
        timeline,
    };

    template
        .render()
        .map(|html| Html(html))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
pub async fn timeline_api(State(pool): State<SqlitePool>) -> Result<Json<Vec<TimelineEvent>>, StatusCode> {
    Ok(Json(get_timeline(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}
pub async fn questions_api(State(pool): State<SqlitePool>) -> Result<Json<Vec<Question>>, StatusCode> {
    Ok(Json(get_questions(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}
async fn get_timeline(pool: &SqlitePool) -> Result<Vec<TimelineEvent>, sqlx::Error> {
    let timeline = sqlx::query_as::<_, TimelineEvent>("SELECT * FROM timeline_events ORDER BY date")
        .fetch_all(pool)
        .await?;
    Ok(timeline)
}
async fn get_questions(pool: &SqlitePool) -> Result<Vec<Question>, sqlx::Error> {
    let questions = sqlx::query_as::<_, Question>("SELECT * FROM questions ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(questions)
}

pub async fn create_question(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewQuestion>,
) -> Result<Json<Question>, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let default_colors = ["yellow", "orange", "pink", "blue", "green"];
    let color = default_colors[rand_index() % default_colors.len()];

    sqlx::query(
        "INSERT INTO questions (id, title, answer, author, color, created_at) VALUES (?, ?, ?, ?, ?, datetime('now'))"
    )
    .bind(&id)
    .bind(&payload.title)
    .bind("待回答...")
    .bind(&payload.author)
    .bind(color)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let question = sqlx::query_as::<_, Question>("SELECT * FROM questions WHERE id = ?")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(question))
}

pub async fn answer_question(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewAnswer>,
) -> Result<(), StatusCode> {
    sqlx::query(
        "UPDATE questions SET answer = ? WHERE id = ?",
    )
        .bind(payload.answer)
        .bind(payload.id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

fn rand_index() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    (seed as usize) % 5
}
