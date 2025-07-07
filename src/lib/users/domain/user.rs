use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UserLoginError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Internal server error")]
    InternalServerError(String),
}
