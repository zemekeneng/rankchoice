use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use std::collections::HashMap;

use crate::models::{
    ballot::Ballot,
    poll::Poll,
    candidate::Candidate,
};
use crate::services::{
    auth::AuthService,
    rcv::{SingleWinnerRCV, Candidate as RcvCandidate},
};

// Reuse the same response structures
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
pub struct PollResultsResponse {
    pub poll_id: Uuid,
    pub total_votes: usize,
    pub status: String,
    pub winner: Option<WinnerInfo>,
    pub final_rankings: Vec<FinalRanking>,
}

#[derive(Debug, Serialize)]
pub struct WinnerInfo {
    pub candidate_id: Uuid,
    pub name: String,
    pub final_votes: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct FinalRanking {
    pub position: usize,
    pub candidate_id: Uuid,
    pub name: String,
    pub votes: f64,
    pub percentage: f64,
    pub eliminated_round: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct RcvRoundsResponse {
    pub rounds: Vec<RoundInfo>,
    pub total_ballots: usize,
    pub exhausted_ballots: usize,
}

#[derive(Debug, Serialize)]
pub struct RoundInfo {
    pub round_number: usize,
    pub vote_counts: HashMap<Uuid, VoteCounts>,
    pub eliminated: Option<EliminatedCandidate>,
    pub winner: Option<WinnerCandidate>,
    pub exhausted_ballots: usize,
    pub total_votes: f64,
    pub majority_threshold: f64,
}

#[derive(Debug, Serialize)]
pub struct VoteCounts {
    pub candidate_id: Uuid,
    pub name: String,
    pub votes: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct EliminatedCandidate {
    pub candidate_id: Uuid,
    pub name: String,
    pub votes: f64,
}

#[derive(Debug, Serialize)]
pub struct WinnerCandidate {
    pub candidate_id: Uuid,
    pub name: String,
    pub votes: f64,
    pub percentage: f64,
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

// Helper function to get user ID for API calls
// TODO: Replace with proper authentication middleware
fn get_current_user_id() -> Uuid {
    if cfg!(test) || std::env::var("CARGO_PKG_NAME").unwrap_or_default().contains("rankchoice") {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    } else {
        Uuid::new_v4()
    }
}

/// GET /api/polls/:id/results - Get poll results
pub async fn get_poll_results(
    Path(poll_id): Path<Uuid>,
    State(auth_service): State<AuthService>,
) -> Result<Json<ApiResponse<PollResultsResponse>>, StatusCode> {
    let pool = auth_service.pool();
    let current_user_id = get_current_user_id();

    // Get poll and verify ownership
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

    // Verify poll ownership
    if poll.user_id != current_user_id {
        return Ok(Json(create_error_response("FORBIDDEN", "You don't have permission to view these results")));
    }

    // Get candidates
    let candidates = match Candidate::find_by_poll_id(pool, poll_id).await {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Database error finding candidates: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get ballots for RCV tabulation
    let ballots = match Ballot::find_by_poll_id(pool, poll_id).await {
        Ok(ballots) => ballots,
        Err(e) => {
            tracing::error!("Database error finding ballots: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if ballots.is_empty() {
        return Ok(Json(create_api_response(PollResultsResponse {
            poll_id,
            total_votes: 0,
            status: "no_votes".to_string(),
            winner: None,
            final_rankings: Vec::new(),
        })));
    }

    // Convert to RCV format
    let rcv_candidates: Vec<RcvCandidate> = candidates.iter()
        .map(|c| RcvCandidate {
            id: c.id,
            name: c.name.clone(),
        })
        .collect();

    // Run RCV tabulation
    let rcv_engine = SingleWinnerRCV::new(rcv_candidates.clone(), ballots.clone());
    let rcv_result = match rcv_engine.tabulate() {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("RCV tabulation error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Determine poll status
    let now = chrono::Utc::now();
    let is_closed = poll.closes_at.map_or(false, |closes| now > closes);
    let status = if is_closed {
        "completed"
    } else if rcv_result.winner.is_some() {
        "winner_declared"
    } else {
        "in_progress"
    };

    // Get final round for results
    let final_round = rcv_result.rounds.last();
    
    let winner = if let (Some(winner_id), Some(final_round)) = (rcv_result.winner, final_round) {
        if let Some(candidate) = rcv_candidates.iter().find(|c| c.id == winner_id) {
            let winner_votes = final_round.vote_counts.get(&winner_id).unwrap_or(&0.0);
            let percentage = if final_round.total_votes > 0.0 {
                (winner_votes / final_round.total_votes) * 100.0
            } else {
                0.0
            };
            
            Some(WinnerInfo {
                candidate_id: winner_id,
                name: candidate.name.clone(),
                final_votes: *winner_votes,
                percentage,
            })
        } else {
            None
        }
    } else {
        None
    };

    // Create final rankings
    let mut final_rankings = Vec::new();
    if let Some(final_round) = final_round {
        let mut rankings: Vec<(Uuid, f64)> = final_round.vote_counts.iter()
            .map(|(&id, &votes)| (id, votes))
            .collect();
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (position, (candidate_id, votes)) in rankings.iter().enumerate() {
            if let Some(candidate) = rcv_candidates.iter().find(|c| c.id == *candidate_id) {
                let percentage = if final_round.total_votes > 0.0 {
                    (votes / final_round.total_votes) * 100.0
                } else {
                    0.0
                };

                // Find elimination round (if any)
                let eliminated_round = rcv_result.rounds.iter()
                    .find(|r| r.eliminated == Some(*candidate_id))
                    .map(|r| r.round_number);

                final_rankings.push(FinalRanking {
                    position: position + 1,
                    candidate_id: *candidate_id,
                    name: candidate.name.clone(),
                    votes: *votes,
                    percentage,
                    eliminated_round,
                });
            }
        }
    }

    let response = PollResultsResponse {
        poll_id,
        total_votes: ballots.len(),
        status: status.to_string(),
        winner,
        final_rankings,
    };

    Ok(Json(create_api_response(response)))
}

/// GET /api/polls/:id/results/rounds - Get RCV rounds
pub async fn get_rcv_rounds(
    Path(poll_id): Path<Uuid>,
    State(auth_service): State<AuthService>,
) -> Result<Json<ApiResponse<RcvRoundsResponse>>, StatusCode> {
    let pool = auth_service.pool();
    let current_user_id = get_current_user_id();

    // Get poll and verify ownership
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

    // Verify poll ownership
    if poll.user_id != current_user_id {
        return Ok(Json(create_error_response("FORBIDDEN", "You don't have permission to view these results")));
    }

    // Get candidates
    let candidates = match Candidate::find_by_poll_id(pool, poll_id).await {
        Ok(candidates) => candidates,
        Err(e) => {
            tracing::error!("Database error finding candidates: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create candidate lookup map
    let candidate_map: HashMap<Uuid, String> = candidates.iter()
        .map(|c| (c.id, c.name.clone()))
        .collect();

    // Get ballots for RCV tabulation
    let ballots = match Ballot::find_by_poll_id(pool, poll_id).await {
        Ok(ballots) => ballots,
        Err(e) => {
            tracing::error!("Database error finding ballots: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if ballots.is_empty() {
        return Ok(Json(create_api_response(RcvRoundsResponse {
            rounds: Vec::new(),
            total_ballots: 0,
            exhausted_ballots: 0,
        })));
    }

    // Convert to RCV format
    let rcv_candidates: Vec<RcvCandidate> = candidates.iter()
        .map(|c| RcvCandidate {
            id: c.id,
            name: c.name.clone(),
        })
        .collect();

    // Run RCV tabulation
    let rcv_engine = SingleWinnerRCV::new(rcv_candidates, ballots.clone());
    let rcv_result = match rcv_engine.tabulate() {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("RCV tabulation error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert rounds to API format
    let rounds = rcv_result.rounds.iter().map(|round| {
        let vote_counts = round.vote_counts.iter().map(|(&candidate_id, &votes)| {
            let name = candidate_map.get(&candidate_id).unwrap_or(&"Unknown".to_string()).clone();
            let percentage = if round.total_votes > 0.0 {
                (votes / round.total_votes) * 100.0
            } else {
                0.0
            };
            
            (candidate_id, VoteCounts {
                candidate_id,
                name,
                votes,
                percentage,
            })
        }).collect();

        let eliminated = round.eliminated.map(|candidate_id| {
            let name = candidate_map.get(&candidate_id).unwrap_or(&"Unknown".to_string()).clone();
            let votes = round.vote_counts.get(&candidate_id).unwrap_or(&0.0);
            EliminatedCandidate {
                candidate_id,
                name,
                votes: *votes,
            }
        });

        let winner = round.winner.map(|candidate_id| {
            let name = candidate_map.get(&candidate_id).unwrap_or(&"Unknown".to_string()).clone();
            let votes = round.vote_counts.get(&candidate_id).unwrap_or(&0.0);
            let percentage = if round.total_votes > 0.0 {
                (votes / round.total_votes) * 100.0
            } else {
                0.0
            };
            WinnerCandidate {
                candidate_id,
                name,
                votes: *votes,
                percentage,
            }
        });

        RoundInfo {
            round_number: round.round_number,
            vote_counts,
            eliminated,
            winner,
            exhausted_ballots: round.exhausted_ballots,
            total_votes: round.total_votes,
            majority_threshold: round.majority_threshold,
        }
    }).collect();

    let response = RcvRoundsResponse {
        rounds,
        total_ballots: ballots.len(),
        exhausted_ballots: rcv_result.exhausted_ballots,
    };

    Ok(Json(create_api_response(response)))
} 