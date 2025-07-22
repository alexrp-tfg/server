use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::users::domain::{Role, User, UserRepository, UserRepositoryError};

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetUserQuery {
    pub id: uuid::Uuid,
}

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct GetUserResult {
    pub id: uuid::Uuid,
    pub username: String,
    pub role: Role,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub async fn get_user_query_handler<UR: UserRepository>(
    query: GetUserQuery,
    user_repository: &UR,
) -> Result<Option<GetUserResult>, UserRepositoryError> {
    let user = user_repository.get_by_id(query.id).await?;
    Ok(user.map(|u| u.into()))
}

impl From<User> for GetUserResult {
    fn from(user: User) -> Self {
        GetUserResult {
            id: user.id,
            username: user.username,
            role: user.role,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
