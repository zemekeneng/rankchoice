use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::user::{CreateUserRequest, LoginRequest};
use crate::services::auth::{AuthError, AuthService};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
    metadata: ApiMetadata,
}

#[derive(Debug, Serialize)]
struct ApiError {
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiMetadata {
    timestamp: String,
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    token: String,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: ApiMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    fn error(code: &str, message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
            }),
            metadata: ApiMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

pub async fn register(
    State(auth_service): State<AuthService>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<crate::services::auth::AuthResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match auth_service.register(req).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(AuthError::UserAlreadyExists) => Err((
            StatusCode::CONFLICT,
            Json(ApiResponse::<()>::error("USER_ALREADY_EXISTS", "A user with this email already exists")),
        )),
        Err(AuthError::Database(e)) => {
            tracing::error!("Database error during registration: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("INTERNAL_ERROR", "Internal server error")),
            ))
        }
        Err(e) => {
            tracing::error!("Registration error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("REGISTRATION_FAILED", &e.to_string())),
            ))
        }
    }
}

pub async fn login(
    State(auth_service): State<AuthService>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<crate::services::auth::AuthResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match auth_service.login(req).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(AuthError::InvalidCredentials) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("INVALID_CREDENTIALS", "Invalid email or password")),
        )),
        Err(AuthError::Database(e)) => {
            tracing::error!("Database error during login: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("INTERNAL_ERROR", "Internal server error")),
            ))
        }
        Err(e) => {
            tracing::error!("Login error: {}", e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("LOGIN_FAILED", &e.to_string())),
            ))
        }
    }
}

pub async fn refresh(
    State(auth_service): State<AuthService>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<RefreshTokenResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match auth_service.refresh_token(&req.refresh_token).await {
        Ok(token) => Ok(Json(ApiResponse::success(RefreshTokenResponse { token }))),
        Err(AuthError::InvalidToken) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("INVALID_TOKEN", "Invalid refresh token")),
        )),
        Err(AuthError::TokenExpired) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("TOKEN_EXPIRED", "Refresh token has expired")),
        )),
        Err(e) => {
            tracing::error!("Token refresh error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("REFRESH_FAILED", "Failed to refresh token")),
            ))
        }
    }
} 