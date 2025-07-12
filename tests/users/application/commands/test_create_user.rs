use lib::users::{application::create_user::{create_user_command_handler, CreateUserCommand}, domain::{user::NewUser, Role, User, UserRepository, UserRepositoryError}};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
struct MockUserRepository {
    user_exists: bool,
    fail_create: bool,
    fail_get: bool,
}

impl UserRepository for MockUserRepository {
    async fn get_by_username(&self, username: String) -> Result<Option<User>, UserRepositoryError> {
        if self.fail_get {
            return Err(UserRepositoryError::InternalServerError);
        }
        if self.user_exists {
            Ok(Some(User {
                id: Uuid::new_v4(),
                username,
                password: "hashed".to_string(),
                role: Role::User,
                created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
                updated_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
            }))
        } else {
            Ok(None)
        }
    }
    async fn create_user(&self, user: NewUser) -> Result<User, UserRepositoryError> {
        if self.fail_create {
            return Err(UserRepositoryError::InternalServerError);
        }
        Ok(User {
            id: Uuid::new_v4(),
            username: user.username,
            password: user.password,
            role: Role::User,
            created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
            updated_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        })
    }
}

#[tokio::test]
async fn test_create_user_success() {
    let repo = MockUserRepository { user_exists: false, fail_create: false, fail_get: false };
    let cmd = CreateUserCommand { username: "alice".to_string(), password: "password123".to_string() };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.username, "alice");
}

#[tokio::test]
async fn test_create_user_already_exists() {
    let repo = MockUserRepository { user_exists: true, fail_create: false, fail_get: false };
    let cmd = CreateUserCommand { username: "bob".to_string(), password: "password123".to_string() };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(matches!(result, Err(UserRepositoryError::UserAlreadyExists)));
}

#[tokio::test]
async fn test_create_user_repo_error_on_get() {
    let repo = MockUserRepository { user_exists: false, fail_create: false, fail_get: true };
    let cmd = CreateUserCommand { username: "bob".to_string(), password: "password123".to_string() };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(matches!(result, Err(UserRepositoryError::InternalServerError)));
}

#[tokio::test]
async fn test_create_user_repo_error_on_create() {
    let repo = MockUserRepository { user_exists: false, fail_create: true, fail_get: false };
    let cmd = CreateUserCommand { username: "bob".to_string(), password: "password123".to_string() };
    let result = create_user_command_handler(cmd, &repo).await;
    assert!(matches!(result, Err(UserRepositoryError::InternalServerError)));
} 
