use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::users::domain::{user::UserLoginError, UserRepository};

#[derive(Debug, ToSchema, Deserialize, Validate)]
pub struct LoginCommand {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Serialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct JWT(
    pub String
);

pub async fn login_command_handler(command: LoginCommand, user_repository: &impl UserRepository) -> Result<JWT, UserLoginError> {
    // Simulate successful login and return a dummy token
    Ok(JWT("dummy".to_string()))
}
