use lib::users::{domain::{user::{User, NewUser, UserLoginError}, Role, UserRepository, UserRepositoryError, Claims, LoginTokenService, Token}};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct MockUserRepository {
    pub user_exists: bool,
    pub fail_create: bool,
    pub fail_get: bool,
    pub user: Option<User>,
}

impl Default for MockUserRepository {
    fn default() -> Self {
        Self {
            user_exists: false,
            fail_create: false,
            fail_get: false,
            user: None,
        }
    }
}

impl UserRepository for MockUserRepository {
    fn get_by_username<'a>(&'a self, username: String) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<User>, UserRepositoryError>> + Send + 'a>> {
        let user_exists = self.user_exists;
        let fail_get = self.fail_get;
        let user = self.user.clone();
        Box::pin(async move {
            if fail_get {
                return Err(UserRepositoryError::InternalServerError);
            }
            if user_exists {
                if let Some(u) = user {
                    Ok(Some(u))
                } else {
                    Ok(Some(User {
                        id: Uuid::new_v4(),
                        username,
                        password: "hashed".to_string(),
                        role: Role::User,
                        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
                        updated_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
                    }))
                }
            } else {
                Ok(None)
            }
        })
    }
    fn create_user<'a>(&'a self, user: NewUser) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<User, UserRepositoryError>> + Send + 'a>> {
        let fail_create = self.fail_create;
        Box::pin(async move {
            if fail_create {
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
                    created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
                    updated_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
                })
            }
        })
    }
}

#[derive(Clone)]
pub struct MockLoginTokenService {
    pub fail: bool,
    pub validation_fail: bool
}

impl Default for MockLoginTokenService {
    fn default() -> Self {
        Self { fail: false, validation_fail: false }
    }
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
            Err(UserLoginError::InternalServerError("validation fail".to_string()))
        } else {
            Ok(Claims {
                sub: Uuid::new_v4().to_string(),
                role: Role::Admin,
                username: "mockuser".to_string(),
                exp: (chrono::Utc::now().timestamp() + 3600) as u64, // 1 hour expiration
            })
        }
    }
} 
