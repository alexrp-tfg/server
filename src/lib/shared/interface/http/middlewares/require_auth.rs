use axum::{
    body::Body,
    extract::State,
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    api::{domain::errors::ApiError, http_server::AppState},
    users::domain::{Claims, Role},
};

pub async fn mw_require_auth(
    State(state): State<AppState>,
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

pub async fn mw_require_role(
    allowed_roles: &'static [Role],
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, Response<Body>> {
    // Extract claims from request extensions (should be set by mw_require_auth)
    println!("Extracting claims from request extensions");
    let claims = req.extensions().get::<Claims>().ok_or_else(|| {
        ApiError::UnauthorizedError("Missing authentication".to_string()).into_response()
    })?;

    // Check if user's role is in the allowed roles
    if allowed_roles.contains(&claims.role) {
        Ok(next.run(req).await)
    } else {
        Err(ApiError::ForbiddenError("Insufficient permissions".to_string()).into_response())
    }
}

// Macro to create protected middleware
#[macro_export]
macro_rules! protected {
    ($state:expr) => {
        axum::middleware::from_fn_with_state(
            $state,
            $crate::shared::interface::http::middlewares::require_auth::mw_require_auth,
        )
    };
}

// Macro to create role-based protected middleware
#[macro_export]
macro_rules! require_roles {
    ($roles:expr) => {
        axum::middleware::from_fn(move |req, next| mw_require_role($roles, req, next))
    };
}
