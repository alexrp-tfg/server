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
    #[schema(examples("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyMjkxNjEyMy0wZjU1LTQyZGItYjJlMy02MTI5ZGEzYTQyOGUiLCJ1c2VybmFtZSI6ImFkbWluIiwiZXhwIjoxNzUxOTk4NDA3fQ.QYqF1DDyRHC-1mptYo7CRRT59T0JiBt8239ZB36Uq0U"))]
    pub token: JWT,
}

impl TokenResponseBody {
    pub fn new(token: JWT) -> Self {
        Self {
            token
        }
    }
}
