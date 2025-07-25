use std::future::Future;

use crate::users::domain::{User, user::NewUser};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn get_by_username(
        &self,
        username: String,
    ) -> impl Future<Output = Result<Option<User>, UserRepositoryError>> + Send;
    fn get_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<User>, UserRepositoryError>> + Send;
    fn get_all_users(&self) -> impl Future<Output = Result<Vec<User>, UserRepositoryError>> + Send;
    fn create_user(
        &self,
        user: NewUser,
    ) -> impl Future<Output = Result<User, UserRepositoryError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum UserRepositoryError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Unexpected error")]
    InternalServerError,
}
