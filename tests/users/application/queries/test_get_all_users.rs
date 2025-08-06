use lib::users::{
    application::queries::get_all_users::{get_all_users_query_handler, GetAllUsersResult},
    domain::{Role, User, UserRepositoryError}
};
use crate::users::MockUserRepository;
use uuid::Uuid;
use chrono::{NaiveDateTime, DateTime};

#[tokio::test]
async fn test_get_all_users_success() {
    let users = vec![
        User {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            password: "hashed1".to_string(),
            role: Role::User,
            created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
            updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        },
        User {
            id: Uuid::new_v4(),
            username: "bob".to_string(),
            password: "hashed2".to_string(),
            role: Role::Admin,
            created_at: Some(DateTime::from_timestamp(1, 0).unwrap().naive_utc()),
            updated_at: Some(DateTime::from_timestamp(1, 0).unwrap().naive_utc()),
        },
    ];
    let repo = MockUserRepository { 
        users_list: users.clone(), 
        fail_get: false,
        ..MockUserRepository::default()
    };
    
    let result = get_all_users_query_handler(&repo).await;
    
    assert!(result.is_ok());
    let users_result = result.unwrap();
    assert_eq!(users_result.len(), 2);
    assert_eq!(users_result[0].username, "alice");
    assert_eq!(users_result[1].username, "bob");
    assert_eq!(users_result[0].id, users[0].id);
    assert_eq!(users_result[1].id, users[1].id);
}

#[tokio::test]
async fn test_get_all_users_empty_list() {
    let repo = MockUserRepository::default();
    
    let result = get_all_users_query_handler(&repo).await;
    
    assert!(result.is_ok());
    let users_result = result.unwrap();
    assert_eq!(users_result.len(), 0);
}

#[tokio::test]
async fn test_get_all_users_repository_error() {
    let repo = MockUserRepository { 
        fail_get: true,
        ..MockUserRepository::default()
    };
    
    let result = get_all_users_query_handler(&repo).await;
    
    assert!(matches!(result, Err(UserRepositoryError::InternalServerError)));
}

#[tokio::test]
async fn test_get_all_users_result_excludes_sensitive_data() {
    let user = User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        password: "secret_password".to_string(),
        role: Role::Admin,
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
    };
    let repo = MockUserRepository { 
        users_list: vec![user.clone()],
        ..MockUserRepository::default()
    };
    
    let result = get_all_users_query_handler(&repo).await;
    
    assert!(result.is_ok());
    let users_result = result.unwrap();
    assert_eq!(users_result.len(), 1);
    let user_result = &users_result[0];
    
    // Verify sensitive fields are excluded
    assert_eq!(user_result.id, user.id);
    assert_eq!(user_result.username, user.username);
    assert_eq!(user_result.created_at, user.created_at);
    assert_eq!(user_result.updated_at, user.updated_at);
    // Password and role should not be accessible in GetAllUsersResult
}
