use async_trait::async_trait;

use crate::users::domain::{User, user::NewUser};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_username(
        &self,
        username: String,
    ) -> Result<Option<User>, UserRepositoryError>;
    async fn get_by_id(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<User>, UserRepositoryError>;
    async fn get_all_users(&self) -> Result<Vec<User>, UserRepositoryError>;
    async fn create_user(
        &self,
        user: NewUser,
    ) -> Result<User, UserRepositoryError>;
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
