use crate::users::MockUserRepository;
use lib::users::{
    application::commands::create_user::{CreateUserCommand, create_user_command_handler},
    domain::UserRepositoryError,
};

#[tokio::test]
async fn test_create_user_success() {
    let repo = MockUserRepository {
        user_exists: false,
        fail_create: false,
        fail_get: false,
        ..MockUserRepository::default()
    };
    let cmd = CreateUserCommand {
        username: "alice".to_string(),
        password: "password123".to_string(),
        role: None,
    };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.username, "alice");
}

#[tokio::test]
async fn test_create_user_already_exists() {
    let repo = MockUserRepository {
        user_exists: true,
        fail_create: false,
        fail_get: false,
        ..MockUserRepository::default()
    };
    let cmd = CreateUserCommand {
        username: "bob".to_string(),
        password: "password123".to_string(),
        role: None,
    };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(matches!(
        result,
        Err(UserRepositoryError::UserAlreadyExists)
    ));
}

#[tokio::test]
async fn test_create_user_repo_error_on_get() {
    let repo = MockUserRepository {
        user_exists: false,
        fail_create: false,
        fail_get: true,
        ..MockUserRepository::default()
    };
    let cmd = CreateUserCommand {
        username: "bob".to_string(),
        password: "password123".to_string(),
        role: None,
    };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(matches!(
        result,
        Err(UserRepositoryError::InternalServerError)
    ));
}

#[tokio::test]
async fn test_create_user_repo_error_on_create() {
    let repo = MockUserRepository {
        user_exists: false,
        fail_create: true,
        fail_get: false,
        ..MockUserRepository::default()
    };
    let cmd = CreateUserCommand {
        username: "bob".to_string(),
        password: "password123".to_string(),
        role: None,
    };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(matches!(
        result,
        Err(UserRepositoryError::InternalServerError)
    ));
}
