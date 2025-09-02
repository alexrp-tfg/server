use axum::{body::Body, extract::{Request, State}, middleware::Next, response::{IntoResponse, Response}};

use crate::api::{domain::errors::ApiError, http_server::AppState};

pub async fn mw_concurrency_semaphore(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Response<Body>> {
    // Extract the JWT token from the Authorization header
    let _permit = state
        .max_concurrent_requests_semaphore
        .acquire()
        .await
        .map_err(|_| { 
            tracing::error!("Failed to acquire semaphore permit");
            ApiError::InternalServerError("Internal server error".to_string()).into_response() 
        })?;

    Ok(next.run(req).await)
}
