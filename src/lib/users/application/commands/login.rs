use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::users::domain::{
    Claims, LoginTokenService, Token, UserRepository,
    user::{UserLogin, UserLoginError},
};

#[derive(Debug, ToSchema, Deserialize, Validate)]
pub struct LoginCommand {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

impl From<LoginCommand> for UserLogin {
    fn from(command: LoginCommand) -> Self {
        UserLogin {
            username: command.username,
            password: command.password,
        }
    }
}

pub async fn login_command_handler(
    command: LoginCommand,
    user_repository: &dyn UserRepository,
    login_token_service: &dyn LoginTokenService,
) -> Result<Token, UserLoginError> {
    let user = (user_repository.get_by_username(command.username).await).unwrap_or_default();

    // Always perform password hashing to prevent timing attacks
    let password_valid = match &user {
        Some(user) => {
            // User exists, verify against stored hash
            let parsed_hash = PasswordHash::new(&user.password)
                .map_err(|_| UserLoginError::InvalidCredentials)?;
            Argon2::default()
                .verify_password(command.password.as_bytes(), &parsed_hash)
                .is_ok()
        }
        None => {
            // User doesn't exist, perform dummy hash verification to maintain consistent timing
            // and prevent timing attacks
            let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$WNVqi0q634KvbTplSaeTjQ$9MDsb3afPzQmWX5pZVVb9/cWjFmdWAqPzQMMX2tomSs";
            let parsed_hash =
                PasswordHash::new(dummy_hash).map_err(|_| UserLoginError::InvalidCredentials)?;
            let _ = Argon2::default().verify_password(command.password.as_bytes(), &parsed_hash);
            // Always return false for non-existent users, regardless of dummy hash result
            false
        }
    };

    // Check if user exists and password is valid
    if let Some(user) = user {
        if password_valid {
            login_token_service.create_token(Claims {
                sub: user.id,
                username: user.username,
                role: user.role,
                exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as u64,
            })
        } else {
            Err(UserLoginError::InvalidCredentials)
        }
    } else {
        Err(UserLoginError::InvalidCredentials)
    }
}
