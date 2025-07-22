use chrono::NaiveDateTime;
use lib::users::{domain::{Role, User}, infrastructure::{models::RowRole, UserRow}};
use uuid::Uuid;

#[test]
fn test_userrow_to_user_from_impl() {
    let row = UserRow {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: "pw".to_string(),
        role: RowRole::User,
        created_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        updated_at: Some(NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
    };
    let user = User::from(row);
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, Role::User);
} 
