use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;

use super::models::{MediaFileModel, NewMediaFileModel};
use crate::media::domain::{MediaFile, MediaRepository, MediaRepositoryError, NewMediaFile};

pub struct DieselMediaRepository {
    connection_pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselMediaRepository {
    pub fn new(connection_pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { connection_pool }
    }
}

#[async_trait]
impl MediaRepository for DieselMediaRepository {
    async fn create_media_file(
        &self,
        media_file: NewMediaFile,
    ) -> Result<MediaFile, MediaRepositoryError> {
        use crate::schema::media_files::dsl::*;

        let new_media_model: NewMediaFileModel = media_file.into();
        let mut conn = self
            .connection_pool
            .get()
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        let created_media = diesel::insert_into(media_files)
            .values(&new_media_model)
            .returning(MediaFileModel::as_returning())
            .get_result(&mut conn)
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        Ok(created_media.into())
    }

    async fn get_media_file_by_id(
        &self,
        media_id: Uuid,
    ) -> Result<Option<MediaFile>, MediaRepositoryError> {
        use crate::schema::media_files::dsl::*;

        let mut conn = self
            .connection_pool
            .get()
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        let result = media_files
            .filter(id.eq(media_id))
            .first::<MediaFileModel>(&mut conn)
            .optional()
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        Ok(result.map(|model| model.into()))
    }

    async fn get_media_files_by_user_id(
        &self,
        user_uuid: Uuid,
    ) -> Result<Vec<MediaFile>, MediaRepositoryError> {
        use crate::schema::media_files::dsl::*;

        let mut conn = self
            .connection_pool
            .get()
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        let results = media_files
            .filter(user_id.eq(user_uuid))
            .order(uploaded_at.desc())
            .load::<MediaFileModel>(&mut conn)
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        Ok(results.into_iter().map(|model| model.into()).collect())
    }

    async fn delete_media_file(&self, media_id: Uuid) -> Result<(), MediaRepositoryError> {
        use crate::schema::media_files::dsl::*;

        let mut conn = self
            .connection_pool
            .get()
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        let deleted_rows = diesel::delete(media_files.filter(id.eq(media_id)))
            .execute(&mut conn)
            .map_err(|_| MediaRepositoryError::InternalServerError)?;

        if deleted_rows == 0 {
            Err(MediaRepositoryError::MediaFileNotFound)
        } else {
            Ok(())
        }
    }
}
