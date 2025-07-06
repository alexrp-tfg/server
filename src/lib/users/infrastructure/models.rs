use diesel::prelude::*;
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
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

#[derive(Insertable, ToSchema, Validate, Deserialize)]
#[diesel(table_name = users)]
// TODO: Define the validation rules for the struct in a separate command file
pub struct CreateUserRow {
    pub username: String,
    pub password: String,
}
