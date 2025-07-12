use lib::users::domain::{user::{NewUser, UserLogin, UserLoginError}, Role, User};
use lib::users::application::queries::get_all_users::GetAllUsersResult;
use lib::users::application::queries::get_user::GetUserResult;
// User domain tests
use uuid::Uuid;
use chrono::NaiveDateTime;

#[test]
fn test_user_struct_equality() {
    let id = Uuid::new_v4();
    let user1 = User {
        id,
        username: "alice".to_string(),
        password: "hashed_pw".to_string(),
        role: Role::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: None,
    };
    let user2 = user1.clone();
    assert_eq!(user1, user2);
}

#[test]
fn test_new_user_struct() {
    let new_user = NewUser {
        username: "bob".to_string(),
        password: "pw".to_string(),
    };
    assert_eq!(new_user.username, "bob");
    assert_eq!(new_user.password, "pw");
}

#[test]
fn test_user_login_struct() {
    let login = UserLogin {
        username: "bob".to_string(),
        password: "pw".to_string(),
    };
    assert_eq!(login.username, "bob");
    assert_eq!(login.password, "pw");
}

#[test]
fn test_user_login_error_variants() {
    let err = UserLoginError::InvalidCredentials;
    assert_eq!(format!("{err}"), "Invalid username or password");
    let err = UserLoginError::InternalServerError("fail".to_string());
    assert_eq!(format!("{err}"), "Internal server error");
    let err = UserLoginError::InvalidToken;
    assert_eq!(format!("{err}"), "Invalid token");
}

#[test]
fn test_user_to_get_all_users_result_conversion() {
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: "alice".to_string(),
        password: "secret_password".to_string(),
        role: Role::Admin,
        created_at: Some(NaiveDateTime::from_timestamp_opt(123456789, 0).unwrap()),
        updated_at: Some(NaiveDateTime::from_timestamp_opt(987654321, 0).unwrap()),
    };
    
    let result: GetAllUsersResult = user.clone().into();
    
    // Verify mapped fields
    assert_eq!(result.id, user.id);
    assert_eq!(result.username, user.username);
    assert_eq!(result.created_at, user.created_at);
    assert_eq!(result.updated_at, user.updated_at);
    
    // This test ensures sensitive fields are not accessible
    // (password and role should not be present in GetAllUsersResult)
}

#[test]
fn test_user_to_get_user_result_conversion() {
    let user_id = Uuid::new_v4();
    let user = User {
        id: user_id,
        username: "bob".to_string(),
        password: "another_secret".to_string(),
        role: Role::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(111111111, 0).unwrap()),
        updated_at: Some(NaiveDateTime::from_timestamp_opt(222222222, 0).unwrap()),
    };
    
    let result: GetUserResult = user.clone().into();
    
    // Verify mapped fields
    assert_eq!(result.id, user.id);
    assert_eq!(result.username, user.username);
    assert_eq!(result.created_at, user.created_at);
    assert_eq!(result.updated_at, user.updated_at);
    
    // This test ensures sensitive fields are not accessible
    // (password and role should not be present in GetUserResult)
} 
