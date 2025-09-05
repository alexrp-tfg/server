use chrono::DateTime;
use lib::users::{
    domain::{Role, User},
    infrastructure::{UserRow, models::RowRole},
};
use uuid::Uuid;

#[test]
fn test_userrow_to_user_from_impl() {
    let row = UserRow {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: "pw".to_string(),
        role: RowRole::User,
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
    };
    let user = User::from(row);
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, Role::User);
}
