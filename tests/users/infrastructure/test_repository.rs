use lib::users::domain::{Role, User, UserRepository, UserRepositoryError};
use crate::users::MockUserRepository;
use uuid::Uuid;
use chrono::{NaiveDateTime, DateTime};

// These tests verify the repository interface behavior using the mock implementation
// Since the DieselUserRepository requires database setup, we test the interface contract

#[tokio::test]
async fn test_repository_get_all_users_interface() {
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
        ..MockUserRepository::default()
    };
    
    let result = repo.get_all_users().await;
    assert!(result.is_ok());
    let returned_users = result.unwrap();
    assert_eq!(returned_users.len(), 2);
    assert_eq!(returned_users[0].username, "alice");
    assert_eq!(returned_users[1].username, "bob");
}

#[tokio::test]
async fn test_repository_get_all_users_empty() {
    let repo = MockUserRepository::default();
    
    let result = repo.get_all_users().await;
    assert!(result.is_ok());
    let returned_users = result.unwrap();
    assert_eq!(returned_users.len(), 0);
}

#[tokio::test]
async fn test_repository_get_all_users_error() {
    let repo = MockUserRepository {
        fail_get: true,
        ..MockUserRepository::default()
    };
    
    let result = repo.get_all_users().await;
    assert!(matches!(result, Err(UserRepositoryError::InternalServerError)));
}

#[tokio::test]
async fn test_repository_get_by_id_interface() {
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: "alice".to_string(),
        password: "hashed".to_string(),
        role: Role::User,
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
    };
    
    let repo = MockUserRepository {
        user: Some(user.clone()),
        ..MockUserRepository::default()
    };
    
    let result = repo.get_by_id(user_id).await;
    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert!(returned_user.is_some());
    let returned_user = returned_user.unwrap();
    assert_eq!(returned_user.id, user_id);
    assert_eq!(returned_user.username, "alice");
}

#[tokio::test]
async fn test_repository_get_by_id_not_found() {
    let repo = MockUserRepository::default();
    let user_id = Uuid::new_v4();
    
    let result = repo.get_by_id(user_id).await;
    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert!(returned_user.is_none());
}

#[tokio::test]
async fn test_repository_get_by_id_error() {
    let repo = MockUserRepository {
        fail_get: true,
        ..MockUserRepository::default()
    };
    let user_id = Uuid::new_v4();
    
    let result = repo.get_by_id(user_id).await;
    assert!(matches!(result, Err(UserRepositoryError::InternalServerError)));
}

#[tokio::test]
async fn test_repository_get_by_id_from_list() {
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
        users_list: users,
        ..MockUserRepository::default()
    };
    
    let result = repo.get_by_id(user_id_2).await;
    assert!(result.is_ok());
    let returned_user = result.unwrap();
    assert!(returned_user.is_some());
    let returned_user = returned_user.unwrap();
    assert_eq!(returned_user.id, user_id_2);
    assert_eq!(returned_user.username, "bob");
}

#[tokio::test]
async fn test_repository_interface_consistency() {
    // Test that all repository methods return consistent error types
    let repo = MockUserRepository {
        fail_get: true,
        fail_create: true,
        ..MockUserRepository::default()
    };
    
    // All methods should return the same error type for consistency
    let get_all_result = repo.get_all_users().await;
    let get_by_id_result = repo.get_by_id(Uuid::new_v4()).await;
    let get_by_username_result = repo.get_by_username("test".to_string()).await;
    
    assert!(matches!(get_all_result, Err(UserRepositoryError::InternalServerError)));
    assert!(matches!(get_by_id_result, Err(UserRepositoryError::InternalServerError)));
    assert!(matches!(get_by_username_result, Err(UserRepositoryError::InternalServerError)));
}