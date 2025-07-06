use diesel::prelude::*;
use std::sync::Arc;

use diesel::result::Error as DieselError;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};

use crate::users::domain::user::NewUser;
use crate::users::infrastructure::{CreateUserRow, UserRow};
use crate::users::user_repository::UserRepositoryError;
use crate::{
    persistence::domain::schema,
    users::domain::{User, UserRepository},
};

#[derive(Clone)]
pub struct DieselUserRepository {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl DieselUserRepository {
    pub fn new(connection: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        DieselUserRepository { pool: connection }
    }
}

impl UserRepository for DieselUserRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, UserRepositoryError> {
        use schema::users::dsl::*;

        // Get a connection from the pool
        let mut conn = self
            .pool
            .get()
            .map_err(|_| UserRepositoryError::InternalServerError)?;

        let created_user = diesel::insert_into(users)
            .values(CreateUserRow::from(new_user))
            .get_result::<UserRow>(&mut *conn)
            .map_err(|e| match e {
                DieselError::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => UserRepositoryError::UserAlreadyExists,
                _ => UserRepositoryError::InternalServerError,
            })?;

        Ok(created_user.into())
    }

    async fn get_by_username(
        &self,
        user_username: String,
    ) -> Result<Option<User>, UserRepositoryError> {
        use schema::users::dsl::*;
        // Get a connection from the pool
        let mut conn = self.pool
            .get()
            .map_err(|_| UserRepositoryError::InternalServerError)?;

        let user_row = users
            .filter(username.eq(user_username))
            .first::<UserRow>(&mut *conn)
            .optional()
            .map_err(|_| UserRepositoryError::InternalServerError)?;

        Ok(user_row.map(User::from))
    }
}
