use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::users::domain::{
        UserRepository,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    username: String,
    exp: u64,
}

#[derive(Serialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct JWT(pub String);

impl JWT {
    pub fn new(logged_user: Claims) -> Result<Self, UserLoginError> {
        match encode(
            &Header::default(),
            &logged_user,
            &EncodingKey::from_secret("secret".as_ref()),
        ) {
            Ok(token) => Ok(JWT(token)),
            Err(_) => Err(UserLoginError::InternalServerError(format!(
                "Failed to generate JWT token"
            ))),
        }
    }
}

pub async fn login_command_handler(
    command: LoginCommand,
    user_repository: &impl UserRepository,
) -> Result<JWT, UserLoginError> {
    let user = match user_repository.get_by_username(command.username).await {
        Ok(user) => user,
        Err(_) => None,
    };

    // Always perform password hashing to prevent timing attacks
    let password_valid = match &user {
        Some(user) => {
            // User exists, verify against stored hash
            let parsed_hash = PasswordHash::new(&user.password).map_err(|_| UserLoginError::InvalidCredentials)?;
            Argon2::default()
                .verify_password(&command.password.as_bytes(), &parsed_hash)
                .is_ok()
        }
        None => {
            // User doesn't exist, perform dummy hash verification to maintain consistent timing
            // and prevent timing attacks
            let dummy_hash = "$argon2id$v=19$m=65536,t=3,p=4$YWJjZGVmZ2hpamtsbW5vcA$kx9KzqFN7eOLCK3xfXx6XWjBhMEoEf0nDdGOJZQVPKo";
            let parsed_hash = PasswordHash::new(dummy_hash).map_err(|_| UserLoginError::InvalidCredentials)?;
            let _ = Argon2::default()
                .verify_password(&command.password.as_bytes(), &parsed_hash);
            // Always return false for non-existent users, regardless of dummy hash result
            false
        }
    };

    // Check if user exists and password is valid
    if let Some(user) = user {
        if password_valid {
            JWT::new(Claims {
                sub: user.id.to_string(),
                username: user.username,
                exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as u64,
            })
        } else {
            Err(UserLoginError::InvalidCredentials)
        }
    } else {
        Err(UserLoginError::InvalidCredentials)
    }
}
