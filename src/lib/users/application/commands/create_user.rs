use argon2::{Argon2, PasswordHasher};
use password_hash::{rand_core::OsRng, SaltString};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
pub struct CreateUserResult {
    pub id: uuid::Uuid,
    pub username: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

pub async fn create_user_command_handler<UR: UserRepository>(
    mut command: CreateUserCommand,
    user_repository: &UR,
) -> Result<CreateUserResult, UserRepositoryError> {

    // Check if the user already exists
    if user_repository.get_by_username(command.username.clone()).await?.is_some() {
        return Err(UserRepositoryError::UserAlreadyExists);
    }

    // Hash user password
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    command.password = argon2
        .hash_password(command.password.as_bytes(), &salt)
        .map_err(|_| UserRepositoryError::InternalServerError)?
        .to_string();

    Ok(user_repository.create_user(command.into()).await?.into())
}

impl From<CreateUserCommand> for NewUser {
    fn from(command: CreateUserCommand) -> Self {
        NewUser {
            username: command.username,
            password: command.password,
        }
    }
}

impl From<User> for CreateUserResult {
    fn from(user: User) -> Self {
        CreateUserResult {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
