use lib::users::{domain::{user::NewUser, Role, User}, infrastructure::{models::RowRole, CreateUserRow, UserRow}};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[test]
fn test_userrow_to_user_mapping() {
    let row = UserRow {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: "pw".to_string(),
        role: RowRole::Admin,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
    };
    let user = User::from(row);
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, Role::Admin);
}

#[test]
fn test_newuser_to_createuserrow_mapping() {
    let new_user = NewUser { username: "bob".to_string(), password: "pw".to_string() };
    let row = CreateUserRow::from(new_user);
    assert_eq!(row.username, "bob");
    assert_eq!(row.password, "pw");
}

#[test]
fn test_rowrole_to_role_mapping() {
    assert_eq!(Role::Admin, RowRole::Admin.into());
    assert_eq!(Role::User, RowRole::User.into());
} 
