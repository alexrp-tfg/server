use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, ToSchema, PartialEq, Eq, Deserialize, Serialize)]
pub enum Role {
    Admin,
    User,
}
