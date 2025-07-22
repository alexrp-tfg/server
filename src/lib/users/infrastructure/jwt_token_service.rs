use std::env;

use jsonwebtoken::{EncodingKey, Header, Validation, encode};
use serde::Serialize;
use utoipa::ToSchema;

use crate::users::domain::{Claims, LoginTokenService, Token, user::UserLoginError};

#[derive(Clone)]
pub struct JwtTokenConfig {
    pub secret_key: String,
}

impl JwtTokenConfig {
    pub fn new() -> Self {
        JwtTokenConfig {
            secret_key: env::var("JWT_SECRET_KEY")
                .expect("JWT_SECRET_KEY environment variable has to be set"),
        }
    }
}

#[derive(Clone)]
pub struct JwtTokenService {
    config: JwtTokenConfig,
}

impl JwtTokenService {
    pub fn new(config: JwtTokenConfig) -> Self {
        JwtTokenService { config }
    }
}

#[derive(Serialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct JWT(pub String);

impl JWT {
    pub fn new(logged_user: Claims, config: &JwtTokenConfig) -> Result<Self, UserLoginError> {
        match encode(
            &Header::default(),
            &logged_user,
            &EncodingKey::from_secret(&config.secret_key.as_ref()),
        ) {
            Ok(token) => Ok(JWT(token)),
            Err(_) => Err(UserLoginError::InternalServerError(format!(
                "Failed to generate JWT token"
            ))),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(token: &str, config: &JwtTokenConfig) -> Result<Claims, UserLoginError> {
        jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(&config.secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| UserLoginError::InvalidToken)
        .map(|data| data.claims)
    }
}

impl LoginTokenService for JwtTokenService {
    fn create_token(
        &self,
        claims: crate::users::domain::Claims,
    ) -> Result<crate::users::domain::Token, UserLoginError> {
        JWT::new(claims, &self.config).map(|jwt| Token(jwt.as_str().to_string()))
    }

    fn validate_token(&self, token: &str) -> Result<Claims, UserLoginError> {
        JWT::validate(token, &self.config)
    }
}
