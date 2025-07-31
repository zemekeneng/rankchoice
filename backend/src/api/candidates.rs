use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use crate::models::candidate::{Candidate, CreateCandidateRequest, UpdateCandidateRequest, ReorderCandidatesRequest};
use crate::services::auth::AuthService;
use crate::api::polls::ApiResponse;

/// Add a new candidate to a poll
pub async fn add_candidate(
    State(auth_service): State<AuthService>,
    Path(poll_id): Path<Uuid>,
    Json(req): Json<CreateCandidateRequest>,
) -> Result<Json<ApiResponse<Candidate>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement proper authentication middleware
    // For now, we'll skip authentication validation

    // Validate request
    if req.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("VALIDATION_ERROR", "Candidate name is required")),
        ));
    }

    match Candidate::create(auth_service.pool(), poll_id, req).await {
        Ok(candidate) => Ok(Json(ApiResponse::success(candidate))),
        Err(e) => {
            tracing::error!("Failed to create candidate: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("CANDIDATE_CREATION_FAILED", "Failed to create candidate")),
            ))
        }
    }
}

/// Update an existing candidate
pub async fn update_candidate(
    State(auth_service): State<AuthService>,
    Path(candidate_id): Path<Uuid>,
    Json(req): Json<UpdateCandidateRequest>,
) -> Result<Json<ApiResponse<Candidate>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement proper authentication middleware
    // For now, we'll skip authentication validation

    // Validate request
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("VALIDATION_ERROR", "Candidate name cannot be empty")),
            ));
        }
    }

    match Candidate::update(auth_service.pool(), candidate_id, req).await {
        Ok(Some(candidate)) => Ok(Json(ApiResponse::success(candidate))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("CANDIDATE_NOT_FOUND", "Candidate not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to update candidate: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("CANDIDATE_UPDATE_FAILED", "Failed to update candidate")),
            ))
        }
    }
}

/// Delete a candidate
pub async fn delete_candidate(
    State(auth_service): State<AuthService>,
    Path(candidate_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement proper authentication middleware
    // For now, we'll skip authentication validation

    match Candidate::delete(auth_service.pool(), candidate_id).await {
        Ok(true) => Ok(Json(ApiResponse::success(()))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("CANDIDATE_NOT_FOUND", "Candidate not found")),
        )),
        Err(e) => {
            tracing::error!("Failed to delete candidate: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("CANDIDATE_DELETE_FAILED", "Failed to delete candidate")),
            ))
        }
    }
}

/// Reorder candidates for a poll
pub async fn reorder_candidates(
    State(auth_service): State<AuthService>,
    Path(poll_id): Path<Uuid>,
    Json(req): Json<ReorderCandidatesRequest>,
) -> Result<Json<ApiResponse<Vec<Candidate>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement proper authentication middleware
    // For now, we'll skip authentication validation

    // Validate request
    if req.candidate_order.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("VALIDATION_ERROR", "At least one candidate ID is required")),
        ));
    }

    match Candidate::reorder(auth_service.pool(), poll_id, req.candidate_order).await {
        Ok(candidates) => Ok(Json(ApiResponse::success(candidates))),
        Err(e) => {
            tracing::error!("Failed to reorder candidates: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("CANDIDATE_REORDER_FAILED", "Failed to reorder candidates")),
            ))
        }
    }
}

/// Get all candidates for a poll
pub async fn list_candidates(
    State(auth_service): State<AuthService>,
    Path(poll_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Candidate>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement proper authentication middleware
    // For now, we'll skip authentication validation

    match Candidate::find_by_poll_id(auth_service.pool(), poll_id).await {
        Ok(candidates) => Ok(Json(ApiResponse::success(candidates))),
        Err(e) => {
            tracing::error!("Failed to list candidates: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("CANDIDATE_LIST_FAILED", "Failed to retrieve candidates")),
            ))
        }
    }
} 