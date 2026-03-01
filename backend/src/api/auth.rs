use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    token: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    message: String,
}

pub async fn verify_email(
    State(auth_service): State<AuthService>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match auth_service.verify_email(&req.token).await {
        Ok(()) => Ok(Json(ApiResponse::success(MessageResponse {
            message: "Email verified successfully".to_string(),
        }))),
        Err(AuthError::InvalidToken) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("INVALID_TOKEN", "Invalid or already used verification token")),
        )),
        Err(AuthError::TokenExpired) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("TOKEN_EXPIRED", "Verification token has expired. Please request a new one.")),
        )),
        Err(e) => {
            tracing::error!("Email verification error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("VERIFICATION_FAILED", "Failed to verify email")),
            ))
        }
    }
}

pub async fn forgot_password(
    State(auth_service): State<AuthService>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Always return success to prevent email enumeration
    if let Err(e) = auth_service.forgot_password(&req.email).await {
        tracing::error!("Forgot password error: {}", e);
    }

    Ok(Json(ApiResponse::success(MessageResponse {
        message: "If an account with that email exists, a password reset link has been sent.".to_string(),
    })))
}

pub async fn reset_password(
    State(auth_service): State<AuthService>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match auth_service.reset_password(&req.token, &req.password).await {
        Ok(()) => Ok(Json(ApiResponse::success(MessageResponse {
            message: "Password has been reset successfully".to_string(),
        }))),
        Err(AuthError::InvalidToken) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("INVALID_TOKEN", "Invalid or already used reset token")),
        )),
        Err(AuthError::TokenExpired) => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("TOKEN_EXPIRED", "Reset token has expired. Please request a new one.")),
        )),
        Err(e) => {
            tracing::error!("Password reset error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("RESET_FAILED", "Failed to reset password")),
            ))
        }
    }
}

pub async fn resend_verification(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MessageResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Extract and verify JWT (same pattern as polls, voters - no middleware)
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", "Missing authorization header")),
            )
        })?;

    let token = auth_header
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.trim().strip_prefix("bearer "))
        .map(str::trim)
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid authorization format")),
            )
        })?;

    let claims = auth_service.verify_token(token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid or expired token")),
        )
    })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("INTERNAL_ERROR", "Invalid user ID")),
        )
    })?;

    match auth_service.resend_verification(user_id).await {
        Ok(()) => Ok(Json(ApiResponse::success(MessageResponse {
            message: "Verification email sent".to_string(),
        }))),
        Err(e) => {
            tracing::error!("Resend verification error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("RESEND_FAILED", "Failed to resend verification email")),
            ))
        }
    }
} 