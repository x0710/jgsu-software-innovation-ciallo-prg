use crate::models::{Answer, NewAnswer, NewQuestion, Question, QuestionAnswers, TimelineEvent};
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
};
use sqlx::SqlitePool;
use tracing::{error, instrument, trace};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    questions: Vec<QuestionAnswers>,
    timeline: Vec<TimelineEvent>,
}

pub async fn index(State(pool): State<SqlitePool>) -> Result<Html<String>, StatusCode> {
    let questions = get_questions(&pool).await.map_err(|e| {
        error!(error=?e, "error getting questions");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let mut timeline = get_timeline(&pool).await.map_err(|e| {
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

pub async fn _timeline_api(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<TimelineEvent>>, StatusCode> {
    Ok(Json(
        get_timeline(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

pub async fn questions_api(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<QuestionAnswers>>, StatusCode> {
    Ok(Json(
        get_questions(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn get_timeline(pool: &SqlitePool) -> Result<Vec<TimelineEvent>, sqlx::Error> {
    let timeline =
        sqlx::query_as::<_, TimelineEvent>("SELECT * FROM timeline_events ORDER BY date")
            .fetch_all(pool)
            .await?;
    Ok(timeline)
}

async fn get_questions(pool: &SqlitePool) -> Result<Vec<QuestionAnswers>, sqlx::Error> {
    let questions =
        sqlx::query_as::<_, Question>("SELECT * FROM questions ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
            .inspect_err(|e| error!(err=?e, "Get questions failed."))?;
    let mut res = Vec::with_capacity(questions.len());
    for question in questions {
        let answers = sqlx::query_as::<_, Answer>(
            "SELECT answer, created_at FROM answers WHERE question_id = ? ORDER BY created_at DESC",
        )
        .bind(&question.id)
        .fetch_all(pool)
        .await
        .inspect_err(|e| error!(err=?e, "Get answers failed."))?;
        res.push(QuestionAnswers { question, answers });
    }
    Ok(res)
}

pub async fn create_question(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewQuestion>,
) -> Result<Json<Question>, StatusCode> {
    let id = uuid::Uuid::new_v4().to_string();
    let default_colors = ["yellow", "orange", "pink", "blue", "green"];
    let color = default_colors[rand_index() % default_colors.len()];

    sqlx::query(
        "INSERT INTO questions (id, title, author, color, created_at) VALUES (?, ?, ?, ?, datetime('now'))"
    )
    .bind(&id)
    .bind(&payload.title)
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

#[instrument(skip_all, fields(q=payload.id))]
pub async fn answer_question(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewAnswer>,
) -> Result<(), StatusCode> {
    trace!(q = payload.id, "Answer Question");
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO answers(id, question_id, answer, created_at) VALUES (?, ?, ?, datetime('now'))",
    )
        .bind(id)
        .bind(payload.id)
        .bind(payload.answer)
        .execute(&pool)
        .await
        .inspect_err(|e| error!(err=?e, "Answer question failed."))
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
