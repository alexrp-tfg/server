use diesel::prelude::*;
use uuid::Uuid;
use crate::persistence::domain::schema::users;

#[derive(Queryable, AsChangeset, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = users)]
pub struct CreateUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
}
