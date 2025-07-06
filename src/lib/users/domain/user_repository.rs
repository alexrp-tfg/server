use std::future::Future;

use crate::users::domain::{user::NewUser, User};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(&self, user: NewUser) -> impl Future<Output = Result<User, UserRepositoryError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum UserRepositoryError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Unexpected error")]
    DatabaseError,
}
