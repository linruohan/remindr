use serde_json::Value;
use sqlx::prelude::FromRow;

use crate::domain::database::document::DocumentModel;
use crate::domain::database::folder::FolderModel;

#[derive(Debug, FromRow)]
pub struct DocumentEntity {
    pub id: i32,
    pub title: String,
    pub content: Value,
    pub folder_id: Option<i32>,
}

impl From<DocumentEntity> for DocumentModel {
    fn from(entity: DocumentEntity) -> Self {
        DocumentModel {
            id: entity.id,
            title: entity.title,
            content: entity.content,
            folder_id: entity.folder_id,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct FolderEntity {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
}

impl From<FolderEntity> for FolderModel {
    fn from(entity: FolderEntity) -> Self {
        FolderModel {
            id: entity.id,
            name: entity.name,
            parent_id: entity.parent_id,
        }
    }
}
