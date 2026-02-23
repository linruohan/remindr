use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct FolderModel {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
}
