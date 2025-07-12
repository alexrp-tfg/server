use lib::users::{application::login::{login_command_handler, LoginCommand}, domain::{user::{NewUser, UserLoginError}, Claims, LoginTokenService, Role, Token, User, UserRepository, UserRepositoryError}};
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::utils::{functions::hash_password};

#[derive(Debug, Clone)]
struct MockUserRepository {
    user: Option<User>,
    fail: bool,
}

impl UserRepository for MockUserRepository {
    async fn get_by_username(&self, _username: String) -> Result<Option<User>, UserRepositoryError> {
        if self.fail {
            return Err(UserRepositoryError::InternalServerError);
        }
        Ok(self.user.clone())
    }
    async fn create_user(&self, _user: NewUser) -> Result<User, UserRepositoryError> {
        unimplemented!()
    }
}

#[derive(Clone)]
struct MockLoginTokenService {
    fail: bool,
}

impl LoginTokenService for MockLoginTokenService {
    fn create_token(&self, _claims: Claims) -> Result<Token, UserLoginError> {
        if self.fail {
            Err(UserLoginError::InternalServerError("fail".to_string()))
        } else {
            Ok(Token("token".to_string()))
        }
    }
    fn validate_token(&self, _token: &str) -> Result<Claims, UserLoginError> {
        unimplemented!()
    }
}


#[tokio::test]
async fn test_login_success() {
    let hashed = hash_password("password123");
    let user = User {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: hashed,
        role: Role::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: None,
    };
    let repo = MockUserRepository { user: Some(user), fail: false };
    let token_service = MockLoginTokenService { fail: false };
    let cmd = LoginCommand { username: "alice".to_string(), password: "password123".to_string() };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "token");
}

#[tokio::test]
async fn test_login_user_not_found() {
    let repo = MockUserRepository { user: None, fail: false };
    let token_service = MockLoginTokenService { fail: false };
    let cmd = LoginCommand { username: "bob".to_string(), password: "password123".to_string() };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(matches!(result, Err(UserLoginError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_wrong_password() {
    let hashed = hash_password("rightpassword");
    let user = User {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: hashed,
        role: Role::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: None,
    };
    let repo = MockUserRepository { user: Some(user), fail: false };
    let token_service = MockLoginTokenService { fail: false };
    let cmd = LoginCommand { username: "alice".to_string(), password: "wrongpassword".to_string() };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(matches!(result, Err(UserLoginError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_invalid_hash() {
    let user = User {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: "not_a_valid_hash".to_string(),
        role: Role::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: None,
    };
    let repo = MockUserRepository { user: Some(user), fail: false };
    let token_service = MockLoginTokenService { fail: false };
    let cmd = LoginCommand { username: "alice".to_string(), password: "password123".to_string() };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(matches!(result, Err(UserLoginError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_token_creation_failure() {
    let hashed = hash_password("password123");
    let user = User {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: hashed,
        role: Role::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: None,
    };
    let repo = MockUserRepository { user: Some(user), fail: false };
    let token_service = MockLoginTokenService { fail: true };
    let cmd = LoginCommand { username: "alice".to_string(), password: "password123".to_string() };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(matches!(result, Err(UserLoginError::InternalServerError(_))));
} 
