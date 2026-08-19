use std::cmp::Ordering;
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

#[derive(Debug, FromRow, Serialize, Clone, Eq, PartialOrd)]
pub struct TimelineEvent {
    pub id: u64,
    pub date: String,
    pub weekday: String,
    pub time: String,
    pub title: String,
    pub event_type: String,
}
impl PartialEq for TimelineEvent {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Ord for TimelineEvent {
    fn cmp(&self, other: &Self) -> Ordering { self.id.cmp(&other.id) }
}

#[derive(Debug, Deserialize)]
pub struct NewQuestion {
    pub title: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct NewAnswer {
    pub id: String,
    pub answer: String,
}
