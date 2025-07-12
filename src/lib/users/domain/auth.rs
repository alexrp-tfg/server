use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::users::domain::{user::UserLoginError, Role};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: Role,
    pub exp: u64,
}

#[derive(Debug, Serialize, ToSchema, Deserialize, Clone, PartialEq, Eq)]
pub struct Token(pub String);

pub trait LoginTokenService: Send + Sync + Clone + 'static {
    fn create_token(&self, claims: Claims) -> Result<Token, UserLoginError>;
    fn validate_token(&self, token: &str) -> Result<Claims, UserLoginError>;
}
