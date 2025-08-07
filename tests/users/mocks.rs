use async_trait::async_trait;
use chrono::DateTime;
use lib::users::domain::{
    Claims, LoginTokenService, Role, Token, UserRepository, UserRepositoryError,
    user::{NewUser, User, UserLoginError},
};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct MockUserRepository {
    pub user_exists: bool,
    pub fail_create: bool,
    pub fail_get: bool,
    pub user: Option<User>,
    pub users_list: Vec<User>,
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn get_by_username(
        &self,
        username: String,
    ) -> Result<Option<lib::users::domain::User>, UserRepositoryError> {
        if self.fail_get {
            return Err(UserRepositoryError::InternalServerError);
        }
        if self.user_exists {
            if let Some(u) = &self.user {
                Ok(Some(u.clone()))
            } else {
                Ok(Some(User {
                    id: Uuid::new_v4(),
                    username,
                    password: "hashed".to_string(),
                    role: Role::User,
                    created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
                    updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
                }))
            }
        } else {
            Ok(None)
        }
    }

    async fn get_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<lib::users::domain::User>, UserRepositoryError> {
        if self.fail_get {
            return Err(UserRepositoryError::InternalServerError);
        }
        if let Some(u) = &self.user {
            if u.id == id {
                return Ok(Some(u.clone()));
            }
        }
        Ok(self.users_list.iter().find(|u| u.id == id).cloned())
    }

    async fn get_all_users(&self) -> Result<Vec<lib::users::domain::User>, UserRepositoryError> {
        if self.fail_get {
            return Err(UserRepositoryError::InternalServerError);
        }
        Ok(self.users_list.clone())
    }

    async fn create_user(
        &self,
        user: NewUser,
    ) -> Result<lib::users::domain::User, UserRepositoryError> {
        if self.fail_create {
            return Err(UserRepositoryError::InternalServerError);
        }
        if user.username == "exists" {
            Err(UserRepositoryError::UserAlreadyExists)
        } else {
            Ok(User {
                id: Uuid::new_v4(),
                username: user.username,
                password: user.password,
                role: Role::User,
                created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
                updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
            })
        }
    }
}

#[derive(Clone, Default)]
pub struct MockLoginTokenService {
    pub fail: bool,
    pub validation_fail: bool,
}

impl LoginTokenService for MockLoginTokenService {
    fn create_token(&self, _claims: Claims) -> Result<Token, UserLoginError> {
        if self.fail {
            Err(UserLoginError::InternalServerError("fail".to_string()))
        } else {
            Ok(Token("mocktoken".to_string()))
        }
    }
    fn validate_token(&self, _token: &str) -> Result<Claims, UserLoginError> {
        if self.validation_fail {
            Err(UserLoginError::InternalServerError(
                "validation fail".to_string(),
            ))
        } else {
            Ok(Claims {
                sub: Uuid::new_v4(),
                role: Role::Admin,
                username: "mockuser".to_string(),
                exp: (chrono::Utc::now().timestamp() + 3600) as u64, // 1 hour expiration
            })
        }
    }
}
