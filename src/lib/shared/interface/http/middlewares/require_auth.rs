use axum::{
    body::Body,
    extract::State,
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    api::{domain::errors::ApiError, http_server::AppState},
    users::domain::{LoginTokenService, UserRepository},
};

pub async fn mw_require_auth<UR: UserRepository, TS: LoginTokenService>(
    State(state): State<AppState<UR, TS>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Response<Body>> {
    // Extract the JWT token from the Authorization header
    let auth_header = req.headers().get("Authorization");

    // TODO: Check if the claim is expired
    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Validate the JWT token
                let claims = state
                    .login_token_service
                    .as_ref()
                    .validate_token(token)
                    .map_err(|_| {
                        ApiError::UnauthorizedError("Unauthorized".to_string()).into_response()
                    })?;
                // If the token is valid, attach the claims to the request
                req.extensions_mut().insert(claims);
                // Proceed to the next middleware or handler
                return Ok(next.run(req).await);
            }
        }
    }

    Err(ApiError::UnauthorizedError("Unauthorized".to_string()).into_response())
}
