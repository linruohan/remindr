use anyhow::Error;
use sqlx::{SqlitePool, query, query_as};

use crate::{domain::database::folder::FolderModel, infrastructure::entities::FolderEntity};

const MAX_FOLDER_DEPTH: u32 = 3;

#[derive(Clone)]
pub struct FolderRepository {
    pool: SqlitePool,
}

impl FolderRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_folders(&self) -> Result<Vec<FolderModel>, Error> {
        query_as::<_, FolderEntity>("SELECT id, name, parent_id FROM folders ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))
            .map(|folders| {
                folders
                    .into_iter()
                    .map(FolderEntity::into)
                    .collect::<Vec<FolderModel>>()
            })
    }

    pub async fn get_folder_by_id(&self, id: i32) -> Result<FolderModel, Error> {
        query_as::<_, FolderEntity>("SELECT id, name, parent_id FROM folders WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map(|r| r.into())
            .map_err(|e| anyhow::Error::from(e))
    }

    pub async fn insert_folder(&self, name: String, parent_id: Option<i32>) -> Result<i32, Error> {
        if let Some(pid) = parent_id {
            let depth = self.compute_depth(pid).await?;
            if depth >= MAX_FOLDER_DEPTH {
                return Err(anyhow::anyhow!(
                    "Cannot create folder: maximum depth of {} reached",
                    MAX_FOLDER_DEPTH
                ));
            }
        }

        let res = query("INSERT INTO folders (name, parent_id) VALUES (?, ?)")
            .bind(&name)
            .bind(parent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        Ok(res.last_insert_rowid() as i32)
    }

    pub async fn update_folder(&self, folder: FolderModel) -> Result<(), Error> {
        query("UPDATE folders SET name = ? WHERE id = ?")
            .bind(&folder.name)
            .bind(folder.id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        Ok(())
    }

    pub async fn delete_folder(&self, id: i32) -> Result<(), Error> {
        query("DELETE FROM folders WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        Ok(())
    }

    pub async fn move_folder(&self, id: i32, new_parent_id: Option<i32>) -> Result<(), Error> {
        if let Some(pid) = new_parent_id {
            let depth = self.compute_depth(pid).await?;
            if depth >= MAX_FOLDER_DEPTH {
                return Err(anyhow::anyhow!(
                    "Cannot move folder: maximum depth of {} would be exceeded",
                    MAX_FOLDER_DEPTH
                ));
            }
        }

        query("UPDATE folders SET parent_id = ? WHERE id = ?")
            .bind(new_parent_id)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        Ok(())
    }

    /// Delete a folder but keep its children by moving them to the folder's parent.
    /// Sub-folders and documents are reparented to parent_id of the deleted folder.
    pub async fn delete_folder_keep_children(&self, id: i32) -> Result<(), Error> {
        let folder = self.get_folder_by_id(id).await?;
        let new_parent = folder.parent_id;

        // Move child folders to the deleted folder's parent
        query("UPDATE folders SET parent_id = ? WHERE parent_id = ?")
            .bind(new_parent)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        // Move child documents to the deleted folder's parent
        query("UPDATE documents SET folder_id = ? WHERE folder_id = ?")
            .bind(new_parent)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::Error::from(e))?;

        // Now delete the empty folder
        self.delete_folder(id).await
    }

    /// Compute the depth of a folder by walking up the parent chain.
    /// Root folders have depth 1, their children depth 2, etc.
    async fn compute_depth(&self, folder_id: i32) -> Result<u32, Error> {
        let mut depth = 1u32;
        let mut current_id = folder_id;

        loop {
            let folder = self.get_folder_by_id(current_id).await?;
            match folder.parent_id {
                Some(pid) => {
                    depth += 1;
                    current_id = pid;
                }
                None => break,
            }
        }

        Ok(depth)
    }
}
