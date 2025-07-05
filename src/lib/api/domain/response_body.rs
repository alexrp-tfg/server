use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Clone, PartialEq, Eq)]
pub struct ApiResponseBody<T: ToSchema + Serialize + Send> {
    pub data: T,
}

impl<T: Serialize + PartialEq + ToSchema + Send> ApiResponseBody<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
        }
    }
}
