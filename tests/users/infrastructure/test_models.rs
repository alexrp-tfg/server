use chrono::DateTime;
use lib::users::{
    domain::{Role, User, user::NewUser},
    infrastructure::{CreateUserRow, UserRow, models::RowRole},
};
use uuid::Uuid;

#[test]
fn test_userrow_to_user_mapping() {
    let row = UserRow {
        id: Uuid::new_v4(),
        username: "alice".to_string(),
        password: "pw".to_string(),
        role: RowRole::Admin,
        created_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
        updated_at: Some(DateTime::from_timestamp(0, 0).unwrap().naive_utc()),
    };
    let user = User::from(row);
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, Role::Admin);
}

#[test]
fn test_newuser_to_createuserrow_mapping() {
    let new_user = NewUser {
        username: "bob".to_string(),
        password: "pw".to_string(),
        role: Some(Role::User),
    };
    let row = CreateUserRow::from(new_user);
    assert_eq!(row.username, "bob");
    assert_eq!(row.password, "pw");
    assert_eq!(row.role.unwrap(), RowRole::User);
}

#[test]
fn test_rowrole_to_role_mapping() {
    assert_eq!(Role::Admin, RowRole::Admin.into());
    assert_eq!(Role::User, RowRole::User.into());
}
