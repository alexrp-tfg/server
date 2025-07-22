// Roles domain tests

use lib::users::domain::Role;

#[test]
fn test_role_enum_serde() {
    let admin = Role::Admin;
    let user = Role::User;
    let admin_json = serde_json::to_string(&admin).unwrap();
    let user_json = serde_json::to_string(&user).unwrap();
    assert_eq!(serde_json::from_str::<Role>(&admin_json).unwrap(), Role::Admin);
    assert_eq!(serde_json::from_str::<Role>(&user_json).unwrap(), Role::User);
} 
