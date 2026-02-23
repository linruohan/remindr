use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DocumentModel {
    pub id: i32,
    pub title: String,
    pub content: Value,
    pub folder_id: Option<i32>,
}
