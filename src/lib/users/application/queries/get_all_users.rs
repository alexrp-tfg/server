use serde::Serialize;
use utoipa::ToSchema;

use crate::users::domain::{User, UserRepository, UserRepositoryError, Role};

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct GetAllUsersResult {
    pub id: uuid::Uuid,
    pub username: String,
    pub role: Role,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub async fn get_all_users_query_handler(
    user_repository: &dyn UserRepository,
) -> Result<Vec<GetAllUsersResult>, UserRepositoryError> {
    let users = user_repository.get_all_users().await?;
    Ok(users.into_iter().map(|user| user.into()).collect())
}

impl From<User> for GetAllUsersResult {
    fn from(user: User) -> Self {
        GetAllUsersResult {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
