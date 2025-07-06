use diesel::prelude::*;
use std::sync::Arc;

use diesel::result::Error as DieselError;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};

use crate::users::infrastructure::{ CreateUserRow, UserRow};
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
    async fn create_user(&self, new_user: &CreateUserRow) -> Result<User, DieselError> {
        use schema::users::dsl::*;

        // Get a connection from the pool
        let mut conn = self.pool.get().map_err(|e| {
            DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let created_user = diesel::insert_into(users)
            .values(new_user)
            .get_result::<UserRow>(&mut *conn)?;

        Ok(created_user.into())
    }
}
