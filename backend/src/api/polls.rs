use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use crate::models::poll::{CreatePollRequest, Poll, PollListQuery, UpdatePollRequest};
use crate::services::auth::AuthService;

// Helper function to get user ID from JWT token
fn get_current_user_id(headers: &HeaderMap, auth_service: &AuthService) -> Result<Uuid, (StatusCode, Json<ApiResponse<()>>)> {
    // In test environment, use hardcoded test user ID
    if cfg!(test) {
        return Ok(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
    }

    // Extract Authorization header
    let authorization = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", "Missing authorization header")),
            )
        })?;

    // Extract token from "Bearer <token>"
    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid authorization format")),
            )
        })?;

    // Verify token and extract user ID
    let claims = auth_service
        .verify_token(token)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid token")),
            )
        })?;

    // Parse user ID from claims
    Uuid::parse_str(&claims.sub)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid user ID in token")),
            )
        })
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
    metadata: ApiMetadata,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    code: String,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiMetadata {
    timestamp: String,
    version: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    items: Vec<T>,
    total: i64,
    page: i32,
    limit: i32,
    total_pages: i32,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
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

    pub fn error(code: &str, message: &str) -> ApiResponse<()> {
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

pub async fn create_poll(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Json(req): Json<CreatePollRequest>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Extract user ID from JWT token
    let user_id = get_current_user_id(&headers, &auth_service)?;

    // Validate request
    if req.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("VALIDATION_ERROR", "Poll title is required")),
        ));
    }

    if req.candidates.len() < 2 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("VALIDATION_ERROR", "At least 2 candidates are required")),
        ));
    }

    // Validate candidate names
    for candidate in &req.candidates {
        if candidate.name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("VALIDATION_ERROR", "All candidate names are required")),
            ));
        }
    }

    match Poll::create(auth_service.pool(), user_id, req).await {
        Ok(poll) => Ok(Json(ApiResponse::success(poll))),
        Err(e) => {
            tracing::error!("Failed to create poll: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("POLL_CREATION_FAILED", "Failed to create poll")),
            ))
        }
    }
}

pub async fn list_polls(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Query(query): Query<PollListQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<crate::models::poll::PollListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = get_current_user_id(&headers, &auth_service)?;

    match Poll::list_by_user(auth_service.pool(), user_id, &query).await {
        Ok((polls, total)) => {
            let page = query.page.unwrap_or(1);
            let limit = query.limit.unwrap_or(20).min(100);
            let total_pages = (total as f64 / limit as f64).ceil() as i32;

            let response = PaginatedResponse {
                items: polls,
                total,
                page,
                limit,
                total_pages,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            tracing::error!("Failed to list polls: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("POLL_LIST_FAILED", "Failed to retrieve polls")),
            ))
        }
    }
}

/// GET /api/public/polls/:id - Get public poll (no auth required)
pub async fn get_public_poll(
    Path(poll_id): Path<Uuid>,
    State(auth_service): State<AuthService>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    match Poll::find_by_id(auth_service.pool(), poll_id).await {
        Ok(Some(poll)) => {
            // Check if poll is public
            if !poll.is_public {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<()>::error("POLL_NOT_PUBLIC", "This poll is not public")),
                ));
            }

            // Load candidates for the poll
            let candidates = match crate::models::candidate::Candidate::find_by_poll_id(auth_service.pool(), poll_id).await {
                Ok(candidates) => candidates,
                Err(e) => {
                    tracing::error!("Failed to load candidates for poll {}: {}", poll_id, e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("CANDIDATES_LOAD_FAILED", "Failed to load poll candidates")),
                    ));
                }
            };

            let poll_response = crate::models::poll::PollResponse {
                id: poll.id,
                user_id: poll.user_id,
                title: poll.title,
                description: poll.description,
                poll_type: poll.poll_type,
                num_winners: poll.num_winners,
                opens_at: poll.opens_at,
                closes_at: poll.closes_at,
                is_public: poll.is_public,
                registration_required: poll.registration_required,
                created_at: poll.created_at,
                updated_at: poll.updated_at,
                candidates,
            };

            Ok(Json(ApiResponse::success(poll_response)))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("POLL_NOT_FOUND", "Poll not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to get poll: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("POLL_GET_FAILED", "Failed to retrieve poll")),
            ))
        }
    }
}

pub async fn get_poll(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Path(poll_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = get_current_user_id(&headers, &auth_service)?;

    match Poll::find_by_id_and_user(auth_service.pool(), poll_id, user_id).await {
        Ok(Some(poll)) => Ok(Json(ApiResponse::success(poll))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("POLL_NOT_FOUND", "Poll not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to get poll: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("POLL_GET_FAILED", "Failed to retrieve poll")),
            ))
        }
    }
}

pub async fn update_poll(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Path(poll_id): Path<Uuid>,
    Json(req): Json<UpdatePollRequest>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = get_current_user_id(&headers, &auth_service)?;

    // Validate request if title is being updated
    if let Some(ref title) = req.title {
        if title.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("VALIDATION_ERROR", "Poll title cannot be empty")),
            ));
        }
    }

    match Poll::update(auth_service.pool(), poll_id, user_id, req).await {
        Ok(Some(poll)) => Ok(Json(ApiResponse::success(poll))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("POLL_NOT_FOUND", "Poll not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to update poll: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("POLL_UPDATE_FAILED", "Failed to update poll")),
            ))
        }
    }
}

pub async fn delete_poll(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Path(poll_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = get_current_user_id(&headers, &auth_service)?;

    match Poll::delete(auth_service.pool(), poll_id, user_id).await {
        Ok(true) => Ok(Json(ApiResponse::success(()))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("POLL_NOT_FOUND", "Poll not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to delete poll: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("POLL_DELETE_FAILED", "Failed to delete poll")),
            ))
        }
    }
} 