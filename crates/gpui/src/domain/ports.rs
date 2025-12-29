use anyhow::Result;
use std::future::Future;

use crate::domain::database::document::DocumentModel;

pub trait DocumentRepositoryPort: Send + Sync {
    type ListFuture<'a>: Future<Output = Result<Vec<DocumentModel>>> + Send + 'a
    where
        Self: 'a;

    /// Future renvoyé par `get`.
    type GetFuture<'a>: Future<Output = Result<Option<DocumentModel>>> + Send + 'a
    where
        Self: 'a;

    type SaveFuture<'a>: Future<Output = Result<()>> + Send + 'a
    where
        Self: 'a;

    fn list<'a>(&'a self) -> Self::ListFuture<'a>;
    fn get<'a>(&'a self, id: i32) -> Self::GetFuture<'a>;
    fn save<'a>(&'a self, document: DocumentModel) -> Self::SaveFuture<'a>;
}
