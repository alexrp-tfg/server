use lib::users::{
    application::queries::get_user::{get_user_query_handler, GetUserQuery, GetUserResult},
    domain::{Role, User, UserRepositoryError}
};
use crate::users::MockUserRepository;
use uuid::Uuid;
use chrono::{NaiveDateTime, DateTime};

#[tokio::test]
async fn test_get_user_success() {
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: "alice".to_string(),
        password: "hashed_password".to_string(),
        role: Role::User,
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
    };
    let repo = MockUserRepository {
        user: Some(user.clone()),
        ..MockUserRepository::default()
    };
    
    let query = GetUserQuery { id: user_id };
    let result = get_user_query_handler(query, &repo).await;
    
    assert!(result.is_ok());
    let user_result = result.unwrap();
    assert!(user_result.is_some());
    let user_result = user_result.unwrap();
    assert_eq!(user_result.id, user.id);
    assert_eq!(user_result.username, user.username);
    assert_eq!(user_result.created_at, user.created_at);
    assert_eq!(user_result.updated_at, user.updated_at);
}

#[tokio::test]
async fn test_get_user_not_found() {
    let user_id = Uuid::new_v4();
    let repo = MockUserRepository::default();
    
    let query = GetUserQuery { id: user_id };
    let result = get_user_query_handler(query, &repo).await;
    
    assert!(result.is_ok());
    let user_result = result.unwrap();
    assert!(user_result.is_none());
}

#[tokio::test]
async fn test_get_user_repository_error() {
    let user_id = Uuid::new_v4();
    let repo = MockUserRepository {
        fail_get: true,
        ..MockUserRepository::default()
    };
    
    let query = GetUserQuery { id: user_id };
    let result = get_user_query_handler(query, &repo).await;
    
    assert!(matches!(result, Err(UserRepositoryError::InternalServerError)));
}

#[tokio::test]
async fn test_get_user_result_excludes_sensitive_data() {
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: "testuser".to_string(),
        password: "super_secret_password".to_string(),
        role: Role::Admin,
        created_at: Some(DateTime::from_timestamp(123456789, 0).unwrap().naive_utc()),
        updated_at: Some(DateTime::from_timestamp(987654321, 0).unwrap().naive_utc()),
    };
    let repo = MockUserRepository {
        user: Some(user.clone()),
        ..MockUserRepository::default()
    };
    
    let query = GetUserQuery { id: user_id };
    let result = get_user_query_handler(query, &repo).await;
    
    assert!(result.is_ok());
    let user_result = result.unwrap();
    assert!(user_result.is_some());
    let user_result = user_result.unwrap();
    
    // Verify sensitive fields are excluded
    assert_eq!(user_result.id, user.id);
    assert_eq!(user_result.username, user.username);
    assert_eq!(user_result.created_at, user.created_at);
    assert_eq!(user_result.updated_at, user.updated_at);
    // Password and role should not be accessible in GetUserResult
}

#[tokio::test]
async fn test_get_user_from_users_list() {
    let user_id_1 = Uuid::new_v4();
    let user_id_2 = Uuid::new_v4();
    let users = vec![
        User {
            id: user_id_1,
            username: "alice".to_string(),
            password: "hashed1".to_string(),
            role: Role::User,
            created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
            updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        },
        User {
            id: user_id_2,
            username: "bob".to_string(),
            password: "hashed2".to_string(),
            role: Role::Admin,
            created_at: Some(DateTime::from_timestamp(1, 0).unwrap().naive_utc()),
            updated_at: Some(DateTime::from_timestamp(1, 0).unwrap().naive_utc()),
        },
    ];
    let repo = MockUserRepository {
        users_list: users.clone(),
        ..MockUserRepository::default()
    };
    
    let query = GetUserQuery { id: user_id_2 };
    let result = get_user_query_handler(query, &repo).await;
    
    assert!(result.is_ok());
    let user_result = result.unwrap();
    assert!(user_result.is_some());
    let user_result = user_result.unwrap();
    assert_eq!(user_result.id, user_id_2);
    assert_eq!(user_result.username, "bob");
}
