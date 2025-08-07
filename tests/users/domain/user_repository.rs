// UserRepositoryError tests

use lib::users::domain::UserRepositoryError;

#[test]
fn test_user_repository_error_variants() {
    let err = UserRepositoryError::UserAlreadyExists;
    assert_eq!(format!("{err}"), "User already exists");
    let err = UserRepositoryError::InternalServerError;
    assert_eq!(format!("{err}"), "Unexpected error");
    let err = UserRepositoryError::UserNotFound;
    assert_eq!(format!("{err}"), "User not found");
}
