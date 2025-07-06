use diesel::prelude::*;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::{persistence::domain::schema::users, users::domain::user::NewUser};

#[derive(Queryable, AsChangeset, Debug)]
#[diesel(table_name = users)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, ToSchema, Deserialize)]
#[diesel(table_name = users)]
pub struct CreateUserRow {
    pub username: String,
    pub password: String,
}

impl From<NewUser> for CreateUserRow {
    fn from(command: NewUser) -> Self {
        CreateUserRow {
            username: command.username,
            password: command.password,
        }
    }
}
