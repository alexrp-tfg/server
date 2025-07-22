use crate::users::{domain::User, infrastructure::UserRow};

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            password: row.password,
            role: row.role.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
