use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::ballot::Voter;
use crate::models::poll::Poll;
use crate::models::user::User;
use crate::services::auth::AuthService;
use crate::services::email::{EmailService, VoterInvitationRequest};

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

fn get_current_user_id(headers: &HeaderMap, auth_service: &AuthService) -> Result<Uuid, (StatusCode, Json<ApiResponse<()>>)> {
    let auth_header = headers.get("authorization").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("UNAUTHORIZED", "Missing authorization header")),
        )
    })?;

    let auth_str = auth_header.to_str().map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid authorization format")),
        )
    })?;

    let token = auth_str.strip_prefix("Bearer ").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid token")),
        )
    })?;

    let claims = auth_service.verify_token(token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid token")),
        )
    })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("UNAUTHORIZED", "Invalid user ID in token")),
        )
    })?;

    Ok(user_id)
}

fn create_api_response<T>(data: T) -> ApiResponse<T> {
    ApiResponse::success(data)
}

fn create_error_response<T>(code: &str, message: &str) -> ApiResponse<T> {
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

#[derive(Debug, Deserialize)]
pub struct CreateVoterRequest {
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VoterResponse {
    pub id: String,
    #[serde(rename = "pollId")]
    pub poll_id: String,
    pub email: Option<String>,
    #[serde(rename = "ballotToken")]
    pub ballot_token: String,
    #[serde(rename = "hasVoted")]
    pub has_voted: bool,
    #[serde(rename = "invitedAt")]
    pub invited_at: String,
    #[serde(rename = "votedAt")]
    pub voted_at: Option<String>,
    #[serde(rename = "votingUrl")]
    pub voting_url: String,
}

#[derive(Debug, Serialize)]
pub struct VotersListResponse {
    pub voters: Vec<VoterResponse>,
    pub total: usize,
    #[serde(rename = "votedCount")]
    pub voted_count: usize,
    #[serde(rename = "pendingCount")]
    pub pending_count: usize,
}

/// POST /api/polls/:id/invite - Create a voter for a poll
pub async fn create_voter(
    Path(poll_id): Path<String>,
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Json(req): Json<CreateVoterRequest>,
) -> Result<Json<ApiResponse<VoterResponse>>, StatusCode> {
    let pool = auth_service.pool();
    
    // Extract user ID from JWT token
    let user_id = match get_current_user_id(&headers, &auth_service) {
        Ok(user_id) => user_id,
        Err((status, _)) => return Err(status),
    };

    // Parse poll ID
    let poll_uuid = match Uuid::parse_str(&poll_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(create_error_response("INVALID_ID", "Invalid poll ID format")));
        }
    };

    // Verify poll exists and user owns it
    let poll = match Poll::find_by_id(pool, poll_uuid).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Poll not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding poll: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if poll.user_id != user_id {
        return Ok(Json(create_error_response("FORBIDDEN", "You don't have permission to manage this poll")));
    }

    // Generate display name for anonymous voters
    let display_email = if req.email.is_none() || req.email.as_ref().map_or(true, |e| e.trim().is_empty()) {
        // Generate a truly unique anonymous voter code using UUID
        Some(format!("Anonymous-{}", Uuid::new_v4()))
    } else {
        req.email
    };

    // Create voter
    let voter = match Voter::create(pool, poll_uuid, display_email, None, None).await {
        Ok(voter) => voter,
        Err(e) => {
            tracing::error!("Database error creating voter: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let voting_url = format!("http://localhost:5173/vote/{}", voter.ballot_token);

    // Send email invitation (if voter has an email)
    if let Some(ref voter_email) = voter.email {
        if !voter_email.starts_with("Anonymous-") {
            // Get poll owner information
            let poll_owner = match User::find_by_id(pool, poll.user_id).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    tracing::warn!("Poll owner not found for poll {}", poll.id);
                    // Continue without sending email
                    User {
                        id: poll.user_id,
                        email: "unknown@rankchoice.app".to_string(),
                        name: Some("Poll Organizer".to_string()),
                        password_hash: String::new(),
                        role: "pollster".to_string(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                }
                Err(e) => {
                    tracing::error!("Database error finding poll owner: {}", e);
                    // Continue without sending email
                    User {
                        id: poll.user_id,
                        email: "unknown@rankchoice.app".to_string(),
                        name: Some("Poll Organizer".to_string()),
                        password_hash: String::new(),
                        role: "pollster".to_string(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                }
            };

            // Create email service and send invitation
            match EmailService::new() {
                Ok(email_service) => {
                    let email_request = VoterInvitationRequest {
                        poll_title: poll.title.clone(),
                        poll_description: poll.description.clone(),
                        voting_url: voting_url.clone(),
                        poll_owner_name: poll_owner.name.unwrap_or_else(|| "Poll Organizer".to_string()),
                        poll_owner_email: poll_owner.email,
                        closes_at: poll.closes_at.map(|dt| dt.to_rfc3339()),
                        voter_name: None, // We could extract this from email if needed
                        to: voter_email.clone(),
                    };

                    match email_service.send_voter_invitation(email_request).await {
                        Ok(email_result) => {
                            if email_result.success {
                                tracing::info!("✅ Email invitation sent to {}", voter_email);
                            } else {
                                tracing::warn!("⚠️ Email service responded with failure for {}: {:?}", 
                                    voter_email, email_result.error);
                            }
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to send email invitation to {}: {}", voter_email, e);
                            // Don't fail the voter creation if email fails
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to create email service: {}", e);
                    // Don't fail the voter creation if email service setup fails
                }
            }
        }
    }

    let response = VoterResponse {
        id: voter.id.to_string(),
        poll_id: voter.poll_id.to_string(),
        email: voter.email.clone(),
        ballot_token: voter.ballot_token.clone(),
        has_voted: voter.has_voted(),
        invited_at: voter.invited_at.to_rfc3339(),
        voted_at: voter.voted_at.map(|dt| dt.to_rfc3339()),
        voting_url,
    };

    Ok(Json(create_api_response(response)))
}

/// GET /api/polls/:id/voters - List voters for a poll
pub async fn list_voters(
    Path(poll_id): Path<String>,
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<VotersListResponse>>, StatusCode> {
    let pool = auth_service.pool();
    
    // Extract user ID from JWT token
    let user_id = match get_current_user_id(&headers, &auth_service) {
        Ok(user_id) => user_id,
        Err((status, _)) => return Err(status),
    };

    // Parse poll ID
    let poll_uuid = match Uuid::parse_str(&poll_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(create_error_response("INVALID_ID", "Invalid poll ID format")));
        }
    };

    // Verify poll exists and user owns it
    let poll = match Poll::find_by_id(pool, poll_uuid).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Poll not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding poll: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if poll.user_id != user_id {
        return Ok(Json(create_error_response("FORBIDDEN", "You don't have permission to view this poll's voters")));
    }

    // Get voters for poll
    let voters = match get_voters_by_poll_id(pool, poll_uuid).await {
        Ok(voters) => voters,
        Err(e) => {
            tracing::error!("Database error finding voters: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let voter_responses: Vec<VoterResponse> = voters
        .iter()
        .map(|voter| {
            let voting_url = format!("http://localhost:5173/vote/{}", voter.ballot_token);
            VoterResponse {
                id: voter.id.to_string(),
                poll_id: voter.poll_id.to_string(),
                email: voter.email.clone(),
                ballot_token: voter.ballot_token.clone(),
                has_voted: voter.has_voted(),
                invited_at: voter.invited_at.to_rfc3339(),
                voted_at: voter.voted_at.map(|dt| dt.to_rfc3339()),
                voting_url,
            }
        })
        .collect();

    let registered_voted_count = voters.iter().filter(|v| v.has_voted()).count();
    
    // Count anonymous ballots (ballots with voter_id = NULL) for this poll
    let anonymous_ballot_count = match sqlx::query!(
        "SELECT COUNT(*) as count FROM ballots WHERE poll_id = $1 AND voter_id IS NULL",
        poll_uuid
    )
    .fetch_one(pool)
    .await {
        Ok(row) => row.count.unwrap_or(0) as usize,
        Err(e) => {
            tracing::error!("Database error counting anonymous ballots: {}", e);
            0
        }
    };
    
    // Total votes = registered voters who voted + anonymous ballots
    let total_voted_count = registered_voted_count + anonymous_ballot_count;
    let pending_count = voters.len() - registered_voted_count; // Only registered voters can be "pending"

    let response = VotersListResponse {
        voters: voter_responses,
        total: voters.len(),
        voted_count: total_voted_count,
        pending_count,
    };

    Ok(Json(create_api_response(response)))
}

/// POST /api/polls/:id/registration - Create a registration link for a poll
pub async fn create_registration_link(
    Path(poll_id): Path<String>,
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<RegistrationLinkResponse>>, StatusCode> {
    let pool = auth_service.pool();
    
    // Extract user ID from JWT token
    let user_id = match get_current_user_id(&headers, &auth_service) {
        Ok(user_id) => user_id,
        Err((status, _)) => return Err(status),
    };

    // Parse poll ID
    let poll_uuid = match Uuid::parse_str(&poll_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(Json(create_error_response("INVALID_ID", "Invalid poll ID format")));
        }
    };

    // Verify poll exists and user owns it
    let poll = match Poll::find_by_id(pool, poll_uuid).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Poll not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding poll: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if poll.user_id != user_id {
        return Ok(Json(create_error_response("FORBIDDEN", "You don't have permission to manage this poll")));
    }

    // Generate a registration token
    let registration_token = format!("reg_{}", Uuid::new_v4());
    
    // Store the registration link in database (you might want to add a registration_links table)
    // For now, we'll return the link directly
    let registration_url = format!("http://localhost:5173/register/{}", registration_token);

    let response = RegistrationLinkResponse {
        poll_id: poll.id.to_string(),
        registration_token,
        registration_url,
        expires_at: None, // You might want to add expiration
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(create_api_response(response)))
}

#[derive(Debug, Serialize)]
pub struct RegistrationLinkResponse {
    #[serde(rename = "pollId")]
    pub poll_id: String,
    #[serde(rename = "registrationToken")]
    pub registration_token: String,
    #[serde(rename = "registrationUrl")]
    pub registration_url: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Helper function to get voters by poll ID
async fn get_voters_by_poll_id(pool: &sqlx::PgPool, poll_id: Uuid) -> Result<Vec<Voter>, sqlx::Error> {
    let voter_rows = sqlx::query!(
        r#"
        SELECT id, poll_id, email, ballot_token, ip_address, user_agent,
               location_data, demographics, invited_at, voted_at
        FROM voters
        WHERE poll_id = $1
        ORDER BY invited_at DESC
        "#,
        poll_id
    )
    .fetch_all(pool)
    .await?;

    let voters = voter_rows
        .into_iter()
        .map(|row| Voter {
            id: row.id,
            poll_id: row.poll_id.expect("poll_id cannot be null"),
            email: row.email,
            ballot_token: row.ballot_token,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            location_data: row.location_data,
            demographics: row.demographics,
            invited_at: row.invited_at.expect("invited_at cannot be null"),
            voted_at: row.voted_at,
        })
        .collect();

    Ok(voters)
} 