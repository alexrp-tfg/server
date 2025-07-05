use crate::users::domain::{user::CreateUser, User};

pub trait UserRepository {
    fn create_user(&self, user: CreateUser) -> Result<User, diesel::result::Error>;
}
