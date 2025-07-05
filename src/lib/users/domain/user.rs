use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, ToSchema, Serialize, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
