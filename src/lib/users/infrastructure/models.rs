use std::io::Write;

use diesel::prelude::*;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use crate::{persistence::domain::schema::{sql_types, users}, users::domain::user::NewUser};

#[derive(Queryable, AsChangeset, Debug, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub role: RowRole,
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = sql_types::Role)]
pub enum RowRole {
    Admin,
    User,
}

impl diesel::serialize::ToSql<sql_types::Role, diesel::pg::Pg> for RowRole {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        match *self {
            RowRole::Admin => out.write_all(b"admin")?,
            RowRole::User => out.write_all(b"user")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}

impl diesel::deserialize::FromSql<sql_types::Role, diesel::pg::Pg> for RowRole {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "ADMIN" => Ok(RowRole::Admin),
            "USER" => Ok(RowRole::User),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<RowRole> for crate::users::domain::Role {
    fn from(row_role: RowRole) -> Self {
        match row_role {
            RowRole::Admin => crate::users::domain::Role::Admin,
            RowRole::User => crate::users::domain::Role::User,
        }
    }
}
