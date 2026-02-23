use crate::infrastructure::repositories::document_repository::DocumentRepository;
use crate::infrastructure::repositories::folder_repository::FolderRepository;
use gpui::Global;

pub struct RepositoryState {
    pub documents: DocumentRepository,
    pub folders: FolderRepository,
}

impl Global for RepositoryState {}
