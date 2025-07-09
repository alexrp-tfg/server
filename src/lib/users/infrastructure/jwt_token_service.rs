use jsonwebtoken::{encode, EncodingKey, Header, Validation};
use serde::Serialize;
use utoipa::ToSchema;

use crate::users::domain::{user::UserLoginError, Claims, LoginTokenService, Token};

#[derive(Clone)]
pub struct JwtTokenService;

#[derive(Serialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct JWT(pub String);

impl JWT {
    pub fn new(logged_user: Claims) -> Result<Self, UserLoginError> {
        // TODO: Use a secure secret key from environment variables or a secure vault
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

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(token: &str) -> Result<Claims, UserLoginError> {
        jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret("secret".as_ref()), &Validation::default())
            .map_err(|_| UserLoginError::InvalidToken)
            .map(|data| data.claims)
}
}

impl LoginTokenService for JwtTokenService {
    fn create_token(&self, claims: crate::users::domain::Claims) -> Result<crate::users::domain::Token, UserLoginError> {
        JWT::new(claims).map(|jwt| Token(jwt.as_str().to_string()))
    }

    fn validate_token(&self, token: &str) -> Result<Claims, UserLoginError> {
        JWT::validate(token)
    }
}
