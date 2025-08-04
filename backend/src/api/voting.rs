use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use ipnetwork::IpNetwork;
use std::net::{IpAddr, SocketAddr};
use axum::extract::ConnectInfo;

use crate::models::{
    ballot::{Ballot, Voter, SubmitBallotRequest, VotingReceiptResponse},
    poll::Poll,
    candidate::Candidate,
};
use crate::services::auth::AuthService;

// Reuse the same response structures from polls.rs
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
pub struct BallotDisplayResponse {
    pub poll: PollForVoting,
    pub voter: VoterStatus,
}

#[derive(Debug, Serialize)]
pub struct PollForVoting {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub poll_type: String,
    pub candidates: Vec<CandidateForVoting>,
    pub is_open: bool,
}

#[derive(Debug, Serialize)]
pub struct CandidateForVoting {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
}

#[derive(Debug, Serialize)]
pub struct VoterStatus {
    pub id: Uuid,
    pub has_voted: bool,
}

#[derive(Debug, Serialize)]
pub struct SubmitBallotResponse {
    pub ballot: BallotSubmissionInfo,
    pub receipt: VotingReceipt,
}

#[derive(Debug, Serialize)]
pub struct BallotSubmissionInfo {
    pub id: Uuid,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct VotingReceipt {
    pub receipt_code: String,
    pub verification_url: String,
}

// Helper functions
fn create_api_response<T>(data: T) -> ApiResponse<T> {
    ApiResponse {
        success: true,
        data: Some(data),
        error: None,
        metadata: ApiMetadata {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    }
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

fn extract_ip_address(connect_info: Option<ConnectInfo<SocketAddr>>) -> Option<IpNetwork> {
    connect_info.map(|info| {
        let ip = info.0.ip();
        match ip {
            IpAddr::V4(ipv4) => IpNetwork::new(IpAddr::V4(ipv4), 32).ok(),
            IpAddr::V6(ipv6) => IpNetwork::new(IpAddr::V6(ipv6), 128).ok(),
        }
    }).flatten()
}

/// GET /api/vote/:token - Get ballot by token
pub async fn get_ballot(
    Path(token): Path<String>,
    State(auth_service): State<AuthService>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
) -> Result<Json<ApiResponse<BallotDisplayResponse>>, StatusCode> {
    let pool = auth_service.pool();

    // Find voter by token
    let voter = match Voter::find_by_token(pool, &token).await {
        Ok(Some(voter)) => voter,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Invalid ballot token")));
        }
        Err(e) => {
            tracing::error!("Database error finding voter: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if voter has already voted
    if voter.has_voted() {
        return Ok(Json(create_error_response("ALREADY_VOTED", "You have already submitted your ballot")));
    }

    // Get poll details
    let poll = match Poll::find_by_id(pool, voter.poll_id).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Poll not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding poll: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if poll is open for voting
    let now = chrono::Utc::now();
    let is_open = poll.opens_at.map_or(true, |opens| now >= opens) &&
                  poll.closes_at.map_or(true, |closes| now <= closes);

    if !is_open {
        return Ok(Json(create_error_response("POLL_CLOSED", "This poll is not currently open for voting")));
    }

    // Get candidates
    let candidates = match Candidate::find_by_poll_id(pool, poll.id).await {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Database error finding candidates: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let poll_for_voting = PollForVoting {
        id: poll.id,
        title: poll.title,
        description: poll.description,
        poll_type: poll.poll_type,
        candidates: candidates.into_iter().map(|c| CandidateForVoting {
            id: c.id,
            name: c.name,
            description: c.description,
            display_order: c.display_order,
        }).collect(),
        is_open,
    };

    let voter_status = VoterStatus {
        id: voter.id,
        has_voted: voter.has_voted(),
    };

    let response = BallotDisplayResponse {
        poll: poll_for_voting,
        voter: voter_status,
    };

    Ok(Json(create_api_response(response)))
}

/// POST /api/vote/:token - Submit ballot
pub async fn submit_ballot(
    Path(token): Path<String>,
    State(auth_service): State<AuthService>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(request): Json<SubmitBallotRequest>,
) -> Result<Json<ApiResponse<SubmitBallotResponse>>, StatusCode> {
    let pool = auth_service.pool();
    let ip_address = extract_ip_address(connect_info);

    // Find voter by token
    let voter = match Voter::find_by_token(pool, &token).await {
        Ok(Some(voter)) => voter,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Invalid ballot token")));
        }
        Err(e) => {
            tracing::error!("Database error finding voter: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if voter has already voted
    if voter.has_voted() {
        return Ok(Json(create_error_response("ALREADY_VOTED", "You have already submitted your ballot")));
    }

    // Get poll to verify it's still open
    let poll = match Poll::find_by_id(pool, voter.poll_id).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Poll not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding poll: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if poll is open for voting
    let now = chrono::Utc::now();
    let is_open = poll.opens_at.map_or(true, |opens| now >= opens) &&
                  poll.closes_at.map_or(true, |closes| now <= closes);

    if !is_open {
        return Ok(Json(create_error_response("POLL_CLOSED", "This poll is not currently open for voting")));
    }

    // Validate ballot rankings
    if request.rankings.is_empty() {
        return Ok(Json(create_error_response("VALIDATION_ERROR", "Ballot must contain at least one ranking")));
    }

    // Verify all candidate IDs belong to this poll
    let candidates = match Candidate::find_by_poll_id(pool, poll.id).await {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Database error finding candidates: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let valid_candidate_ids: std::collections::HashSet<Uuid> = candidates.iter().map(|c| c.id).collect();
    
    for ranking in &request.rankings {
        if !valid_candidate_ids.contains(&ranking.candidate_id) {
            return Ok(Json(create_error_response("VALIDATION_ERROR", "Invalid candidate ID in ballot")));
        }
    }

    // Validate ranking sequence (should be 1, 2, 3, etc.)
    let mut ranks: Vec<i32> = request.rankings.iter().map(|r| r.rank).collect();
    ranks.sort();
    for (i, &rank) in ranks.iter().enumerate() {
        if rank != (i + 1) as i32 {
            return Ok(Json(create_error_response("VALIDATION_ERROR", "Rankings must be sequential starting from 1")));
        }
    }

    // Create ballot with rankings
    let ballot_response = match Ballot::create(pool, voter.id, poll.id, request.rankings, ip_address).await {
        Ok(ballot) => ballot,
        Err(e) => {
            tracing::error!("Database error creating ballot: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Mark voter as having voted
    if let Err(e) = Voter::mark_as_voted(pool, voter.id).await {
        tracing::error!("Database error marking voter as voted: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Generate receipt
    let receipt_code = format!("VOTE-{}-{}", 
        chrono::Utc::now().format("%Y"),
        ballot_response.ballot.id.to_string().split('-').next().unwrap_or("UNKNOWN")
    );
    
    let verification_url = format!("https://rankchoice.app/verify/{}", receipt_code);

    let response = SubmitBallotResponse {
        ballot: BallotSubmissionInfo {
            id: ballot_response.ballot.id,
            submitted_at: ballot_response.ballot.submitted_at,
        },
        receipt: VotingReceipt {
            receipt_code,
            verification_url,
        },
    };

    Ok(Json(create_api_response(response)))
}

/// GET /api/vote/:token/receipt - Get voting receipt
pub async fn get_voting_receipt(
    Path(token): Path<String>,
    State(auth_service): State<AuthService>,
) -> Result<Json<ApiResponse<VotingReceiptResponse>>, StatusCode> {
    let pool = auth_service.pool();

    // Find voter by token
    let voter = match Voter::find_by_token(pool, &token).await {
        Ok(Some(voter)) => voter,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Invalid ballot token")));
        }
        Err(e) => {
            tracing::error!("Database error finding voter: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if voter has voted
    if !voter.has_voted() {
        return Ok(Json(create_error_response("NOT_VOTED", "No ballot has been submitted for this token")));
    }

    // Find the ballot for this voter
    let ballot_query = sqlx::query!(
        "SELECT id, submitted_at FROM ballots WHERE voter_id = $1",
        voter.id
    );

    let ballot_row = match ballot_query.fetch_one(pool).await {
        Ok(row) => row,
        Err(sqlx::Error::RowNotFound) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Ballot not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding ballot: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Generate receipt code (same format as submission)
    let receipt_code = format!("VOTE-{}-{}", 
        ballot_row.submitted_at.expect("submitted_at cannot be null").format("%Y"),
        ballot_row.id.to_string().split('-').next().unwrap_or("UNKNOWN")
    );
    
    let verification_url = format!("https://rankchoice.app/verify/{}", receipt_code);

    let response = VotingReceiptResponse {
        ballot_id: ballot_row.id,
        submitted_at: ballot_row.submitted_at.expect("submitted_at cannot be null"),
        poll_id: voter.poll_id,
        receipt_code,
        verification_url,
    };

    Ok(Json(create_api_response(response)))
}

// Anonymous voting structures
#[derive(Debug, Deserialize)]
pub struct AnonymousVoteRequest {
    pub rankings: Vec<AnonymousRanking>,
}

#[derive(Debug, Deserialize)]
pub struct AnonymousRanking {
    pub candidate_id: Uuid,
    pub rank: i32,
}

#[derive(Debug, Serialize)]
pub struct AnonymousVoteResponse {
    pub ballot: AnonymousBallotInfo,
    pub receipt: VotingReceipt,
}

#[derive(Debug, Serialize)]
pub struct AnonymousBallotInfo {
    pub id: Uuid,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/public/polls/:id/vote - Submit anonymous vote for public poll
pub async fn submit_anonymous_vote(
    Path(poll_id): Path<Uuid>,
    State(auth_service): State<AuthService>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(request): Json<AnonymousVoteRequest>,
) -> Result<Json<ApiResponse<AnonymousVoteResponse>>, StatusCode> {
    let pool = auth_service.pool();
    let ip_address = extract_ip_address(connect_info);

    // Get poll and verify it's public and open
    let poll = match Poll::find_by_id(pool, poll_id).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return Ok(Json(create_error_response("NOT_FOUND", "Poll not found")));
        }
        Err(e) => {
            tracing::error!("Database error finding poll: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Verify poll is public
    if !poll.is_public {
        return Ok(Json(create_error_response("POLL_NOT_PUBLIC", "This poll is not open for public voting")));
    }

    // Check if poll is open for voting
    let now = chrono::Utc::now();
    let is_open = poll.opens_at.map_or(true, |opens| now >= opens) &&
                  poll.closes_at.map_or(true, |closes| now <= closes);

    if !is_open {
        return Ok(Json(create_error_response("POLL_CLOSED", "This poll is not currently open for voting")));
    }

    // Validate ballot rankings
    if request.rankings.is_empty() {
        return Ok(Json(create_error_response("VALIDATION_ERROR", "Ballot must contain at least one ranking")));
    }

    // Verify all candidate IDs belong to this poll
    let candidates = match Candidate::find_by_poll_id(pool, poll_id).await {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Database error finding candidates: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let valid_candidate_ids: std::collections::HashSet<Uuid> = candidates.iter().map(|c| c.id).collect();
    
    for ranking in &request.rankings {
        if !valid_candidate_ids.contains(&ranking.candidate_id) {
            return Ok(Json(create_error_response("VALIDATION_ERROR", "Invalid candidate ID in ballot")));
        }
    }

    // Validate ranking sequence (should be 1, 2, 3, etc.)
    let mut ranks: Vec<i32> = request.rankings.iter().map(|r| r.rank).collect();
    ranks.sort();
    for (i, &rank) in ranks.iter().enumerate() {
        if rank != (i + 1) as i32 {
            return Ok(Json(create_error_response("VALIDATION_ERROR", "Rankings must be sequential starting from 1")));
        }
    }

    // Convert anonymous rankings to ballot rankings
    let ballot_rankings: Vec<crate::models::ballot::BallotRanking> = request.rankings.iter().map(|r| {
        crate::models::ballot::BallotRanking {
            candidate_id: r.candidate_id,
            rank: r.rank,
        }
    }).collect();

    // Create anonymous ballot (without voter_id)
    let ballot_response = match create_anonymous_ballot(pool, poll_id, ballot_rankings, ip_address).await {
        Ok(ballot) => ballot,
        Err(e) => {
            tracing::error!("Database error creating anonymous ballot: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Generate receipt
    let receipt_code = format!("ANON-{}-{}", 
        chrono::Utc::now().format("%Y"),
        ballot_response.id.to_string().split('-').next().unwrap_or("UNKNOWN")
    );
    
    let verification_url = format!("https://rankchoice.app/verify/{}", receipt_code);

    let response = AnonymousVoteResponse {
        ballot: AnonymousBallotInfo {
            id: ballot_response.id,
            submitted_at: ballot_response.submitted_at,
        },
        receipt: VotingReceipt {
            receipt_code,
            verification_url,
        },
    };

    tracing::info!("Anonymous vote submitted for poll {} with ballot ID {}", poll_id, ballot_response.id);

    Ok(Json(create_api_response(response)))
}

// Helper function to create anonymous ballot
async fn create_anonymous_ballot(
    pool: &sqlx::PgPool,
    poll_id: Uuid,
    rankings: Vec<crate::models::ballot::BallotRanking>,
    ip_address: Option<IpNetwork>,
) -> Result<AnonymousBallotInfo, sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Create ballot without voter_id (NULL)
    let ballot_row = sqlx::query!(
        r#"
        INSERT INTO ballots (poll_id, voter_id, ip_address, submitted_at)
        VALUES ($1, NULL, $2, NOW())
        RETURNING id, submitted_at
        "#,
        poll_id,
        ip_address
    )
    .fetch_one(&mut *tx)
    .await?;

    // Insert rankings
    for ranking in rankings {
        sqlx::query!(
            r#"
            INSERT INTO rankings (ballot_id, candidate_id, rank)
            VALUES ($1, $2, $3)
            "#,
            ballot_row.id,
            ranking.candidate_id,
            ranking.rank
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(AnonymousBallotInfo {
        id: ballot_row.id,
        submitted_at: ballot_row.submitted_at.expect("submitted_at cannot be null"),
    })
} 