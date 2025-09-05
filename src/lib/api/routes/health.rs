use axum::Json;
use serde_json::json;

#[utoipa::path(
    get,
    path = "/healthz",
    description = "Health Check Endpoint",
    responses(
        (status = 200, description = "Health Check Successful", body = String, example = "<h1>API is healthy</h1>"),
    )
)]
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy"
    }))
}

pub mod swagger {
    use super::*;
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(health_check))]
    pub struct ApiDoc;
}
