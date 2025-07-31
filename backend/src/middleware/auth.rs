use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;

use crate::services::auth::{AuthService, Claims};

#[derive(Clone)]
pub struct CurrentUser {
    pub claims: Claims,
}

pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract Authorization header
    let authorization = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "Missing authorization header"
                    }
                })),
            )
        })?;

    // Extract token from "Bearer <token>"
    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "Invalid authorization format"
                    }
                })),
            )
        })?;

    // Verify token
    let claims = auth_service.verify_token(token).map_err(|e| {
        let error_message = match e {
            crate::services::auth::AuthError::InvalidToken => "Invalid token",
            crate::services::auth::AuthError::TokenExpired => "Token expired",
            _ => "Authentication failed",
        };
        
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": {
                    "code": "UNAUTHORIZED",
                    "message": error_message
                }
            })),
        )
    })?;

    // Add current user to request extensions
    request.extensions_mut().insert(CurrentUser { claims });

    Ok(next.run(request).await)
}

// Helper to extract current user from request
pub fn extract_current_user(request: &Request) -> Option<&CurrentUser> {
    request.extensions().get::<CurrentUser>()
} 