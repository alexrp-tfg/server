use axum::response::Html;

#[utoipa::path(
    get,
    path = "/health",
    description = "Health Check Endpoint",
    responses(
        (status = 200, description = "Health Check Successful", body = String, example = "<h1>API is healthy</h1>"),
    )
)]
pub async fn health_check() -> Html<&'static str> {
    Html("<h1>API is healthy</h1>")
}

pub mod swagger {
    use super::*;
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(
        paths(health_check),
    )]
    pub struct ApiDoc;
}
