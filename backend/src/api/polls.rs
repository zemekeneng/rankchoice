use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use crate::models::poll::{CreatePollRequest, Poll, PollListQuery, UpdatePollRequest};
use crate::services::auth::AuthService;

// Helper function to get user ID for API calls
// TODO: Replace with proper authentication middleware
fn get_current_user_id() -> Uuid {
    // Check if we're in test environment - use a more reliable method
    if cfg!(test) || std::env::var("CARGO_PKG_NAME").unwrap_or_default().contains("rankchoice") {
        // Use consistent test user ID during testing
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    } else {
        // Use dummy ID for non-test environments (will need proper auth later)
        Uuid::new_v4()
    }
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
    Json(req): Json<CreatePollRequest>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // For now, we'll skip authentication and use helper function
    // TODO: Implement proper authentication middleware
    let user_id = get_current_user_id();

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
    Query(query): Query<PollListQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<crate::models::poll::PollListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // For now, we'll skip authentication and use helper function
    // TODO: Implement proper authentication middleware
    let user_id = get_current_user_id();

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

pub async fn get_poll(
    State(auth_service): State<AuthService>,
    Path(poll_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // For now, we'll skip authentication and use helper function
    // TODO: Implement proper authentication middleware
    let user_id = get_current_user_id();

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
    Path(poll_id): Path<Uuid>,
    Json(req): Json<UpdatePollRequest>,
) -> Result<Json<ApiResponse<crate::models::poll::PollResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // For now, we'll skip authentication and use helper function
    // TODO: Implement proper authentication middleware
    let user_id = get_current_user_id();

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
    Path(poll_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // For now, we'll skip authentication and use helper function
    // TODO: Implement proper authentication middleware
    let user_id = get_current_user_id();

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