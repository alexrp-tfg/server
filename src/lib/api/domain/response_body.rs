use serde::Serialize;
use utoipa::ToSchema;

use crate::users::application::login::JWT;

#[derive(Debug, ToSchema, Serialize, Clone, PartialEq, Eq)]
pub struct ApiResponseBody<T: ToSchema + Serialize + Send> {
    pub data: T,
}

impl<T: Serialize + PartialEq + ToSchema + Send> ApiResponseBody<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
        }
    }
}

#[derive(Debug, ToSchema, Serialize, Clone, PartialEq, Eq)]
pub struct TokenResponseBody {
    pub token: JWT,
}

impl TokenResponseBody {
    pub fn new(token: JWT) -> Self {
        Self {
            token
        }
    }
}
