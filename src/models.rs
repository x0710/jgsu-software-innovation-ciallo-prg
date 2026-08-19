use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Question {
    pub id: String,
    pub title: String,
    pub answer: String,
    pub author: String,
    pub color: String,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct TimelineEvent {
    pub id: String,
    pub date: String,
    pub weekday: String,
    pub time: String,
    pub title: String,
    pub event_type: String,
}

#[derive(Debug, Deserialize)]
pub struct NewQuestion {
    pub title: String,
    pub author: String,
}
