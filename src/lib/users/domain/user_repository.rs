use std::future::Future;

use crate::users::{domain::User, infrastructure::CreateUserRow};

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(&self, user: &CreateUserRow) -> impl Future<Output = Result<User, diesel::result::Error>> + Send;
}
