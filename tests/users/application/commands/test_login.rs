use chrono::DateTime;
use lib::users::{
    application::login::{LoginCommand, login_command_handler},
    domain::{Role, User, user::UserLoginError},
};
use uuid::Uuid;

use crate::{
    users::{MockLoginTokenService, MockUserRepository},
    utils::functions::hash_password,
};

#[tokio::test]
async fn test_login_success() {
    let hashed = hash_password("password123");
    let user = User {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: hashed,
        role: Role::User,
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: None,
    };
    let repo = MockUserRepository {
        user: Some(user),
        user_exists: true,
        ..MockUserRepository::default()
    };
    let token_service = MockLoginTokenService {
        fail: false,
        validation_fail: false,
    };
    let cmd = LoginCommand {
        username: "alice".to_string(),
        password: "password123".to_string(),
    };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "mocktoken");
}

#[tokio::test]
async fn test_login_user_not_found() {
    let repo = MockUserRepository {
        user: None,
        user_exists: false,
        ..MockUserRepository::default()
    };
    let token_service = MockLoginTokenService::default();
    let cmd = LoginCommand {
        username: "bob".to_string(),
        password: "password123".to_string(),
    };
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
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: None,
    };
    let repo = MockUserRepository {
        user: Some(user),
        user_exists: true,
        ..MockUserRepository::default()
    };
    let token_service = MockLoginTokenService::default();
    let cmd = LoginCommand {
        username: "alice".to_string(),
        password: "wrongpassword".to_string(),
    };
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
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: None,
    };
    let repo = MockUserRepository {
        user: Some(user),
        user_exists: true,
        ..MockUserRepository::default()
    };
    let token_service = MockLoginTokenService::default();
    let cmd = LoginCommand {
        username: "alice".to_string(),
        password: "password123".to_string(),
    };
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
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: None,
    };
    let repo = MockUserRepository {
        user: Some(user),
        user_exists: true,
        ..MockUserRepository::default()
    };
    let token_service = MockLoginTokenService {
        fail: true,
        validation_fail: false,
    };
    let cmd = LoginCommand {
        username: "alice".to_string(),
        password: "password123".to_string(),
    };
    let result = login_command_handler(cmd, &repo, &token_service).await;
    assert!(matches!(
        result,
        Err(UserLoginError::InternalServerError(_))
    ));
}
