use diesel::prelude::*;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::persistence::domain::schema::users;

#[derive(Queryable, AsChangeset, Debug)]
#[diesel(table_name = users)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Deserialize, ToSchema)]
#[diesel(table_name = users)]
pub struct CreateUserRow {
    pub username: String,
    pub password: String,
}
