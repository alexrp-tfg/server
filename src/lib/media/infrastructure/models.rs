use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::media_files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MediaFileModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub file_path: String,
    pub uploaded_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::media_files)]
pub struct NewMediaFileModel {
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub file_path: String,
}