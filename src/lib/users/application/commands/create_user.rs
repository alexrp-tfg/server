use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateUserCommand {
    #[validate(email)]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}
