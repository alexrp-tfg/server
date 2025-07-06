use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::users::domain::{User, UserRepository, UserRepositoryError, user::NewUser};

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateUserCommand {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

pub async fn create_user_command_handler<UR: UserRepository>(
    command: CreateUserCommand,
    user_repository: &UR,
) -> Result<User, UserRepositoryError> {
    user_repository.create_user(command.into()).await
}

impl From<CreateUserCommand> for NewUser {
    fn from(command: CreateUserCommand) -> Self {
        NewUser {
            username: command.username,
            password: command.password,
        }
    }
}
