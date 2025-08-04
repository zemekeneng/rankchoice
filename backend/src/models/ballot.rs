use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use ipnetwork::IpNetwork;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Ballot {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub poll_id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub ip_address: Option<IpNetwork>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Ranking {
    pub id: Uuid,
    pub ballot_id: Uuid,
    pub candidate_id: Uuid,
    pub rank: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Voter {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub email: Option<String>,
    pub ballot_token: String,
    pub ip_address: Option<IpNetwork>,
    pub user_agent: Option<String>,
    pub location_data: Option<serde_json::Value>,
    pub demographics: Option<serde_json::Value>,
    pub invited_at: DateTime<Utc>,
    pub voted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitBallotRequest {
    pub rankings: Vec<BallotRanking>,
}

#[derive(Debug, Deserialize)]
pub struct BallotRanking {
    pub candidate_id: Uuid,
    pub rank: i32,
}

#[derive(Debug, Serialize)]
pub struct BallotResponse {
    pub ballot: Ballot,
    pub rankings: Vec<Ranking>,
}

#[derive(Debug, Serialize)]
pub struct VotingReceiptResponse {
    pub ballot_id: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub poll_id: Uuid,
    pub receipt_code: String,
    pub verification_url: String,
}

impl Ballot {
    /// Create a new ballot with rankings
    pub async fn create(
        pool: &PgPool,
        voter_id: Uuid,
        poll_id: Uuid,
        rankings: Vec<BallotRanking>,
        ip_address: Option<IpNetwork>,
    ) -> Result<BallotResponse, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Create the ballot
        let ballot_row = sqlx::query!(
            r#"
            INSERT INTO ballots (voter_id, poll_id, ip_address)
            VALUES ($1, $2, $3)
            RETURNING id, voter_id, poll_id, submitted_at, ip_address
            "#,
            voter_id,
            poll_id,
            ip_address
        )
        .fetch_one(&mut *tx)
        .await?;
        
        let ballot = Ballot {
            id: ballot_row.id,
            voter_id: ballot_row.voter_id.expect("voter_id cannot be null"),
            poll_id: ballot_row.poll_id.expect("poll_id cannot be null"),
            submitted_at: ballot_row.submitted_at.expect("submitted_at cannot be null"),
            ip_address: ballot_row.ip_address,
        };

        // Create the rankings
        let mut created_rankings = Vec::new();
        for ranking in rankings {
            let ranking_row = sqlx::query!(
                r#"
                INSERT INTO rankings (ballot_id, candidate_id, rank)
                VALUES ($1, $2, $3)
                RETURNING id, ballot_id, candidate_id, rank
                "#,
                ballot.id,
                ranking.candidate_id,
                ranking.rank
            )
            .fetch_one(&mut *tx)
            .await?;
            
            let created_ranking = Ranking {
                id: ranking_row.id,
                ballot_id: ranking_row.ballot_id.expect("ballot_id cannot be null"),
                candidate_id: ranking_row.candidate_id.expect("candidate_id cannot be null"),
                rank: ranking_row.rank,
            };
            
            created_rankings.push(created_ranking);
        }

        tx.commit().await?;

        Ok(BallotResponse {
            ballot,
            rankings: created_rankings,
        })
    }

    /// Find ballot by ID with rankings
    pub async fn find_by_id(pool: &PgPool, ballot_id: Uuid) -> Result<Option<BallotResponse>, sqlx::Error> {
        let ballot_row = sqlx::query!(
            "SELECT id, voter_id, poll_id, submitted_at, ip_address FROM ballots WHERE id = $1",
            ballot_id
        )
        .fetch_optional(pool)
        .await?;

        match ballot_row {
            Some(row) => {
                let ballot = Ballot {
                    id: row.id,
                    voter_id: row.voter_id.expect("voter_id cannot be null"),
                    poll_id: row.poll_id.expect("poll_id cannot be null"),
                    submitted_at: row.submitted_at.expect("submitted_at cannot be null"),
                    ip_address: row.ip_address,
                };
                
                let ranking_rows = sqlx::query!(
                    "SELECT id, ballot_id, candidate_id, rank FROM rankings WHERE ballot_id = $1 ORDER BY rank",
                    ballot.id
                )
                .fetch_all(pool)
                .await?;
                
                let rankings = ranking_rows.into_iter().map(|row| Ranking {
                    id: row.id,
                    ballot_id: row.ballot_id.expect("ballot_id cannot be null"),
                    candidate_id: row.candidate_id.expect("candidate_id cannot be null"),
                    rank: row.rank,
                }).collect();

                Ok(Some(BallotResponse { ballot, rankings }))
            }
            None => Ok(None),
        }
    }

    /// Get all ballots for a poll (for RCV tabulation)
    pub async fn find_by_poll_id(pool: &PgPool, poll_id: Uuid) -> Result<Vec<crate::services::rcv::Ballot>, sqlx::Error> {
        let ballot_data = sqlx::query!(
            r#"
            SELECT 
                b.id,
                b.voter_id,
                array_agg(r.candidate_id ORDER BY r.rank) as candidate_ids
            FROM ballots b
            JOIN rankings r ON b.id = r.ballot_id
            WHERE b.poll_id = $1
            GROUP BY b.id, b.voter_id
            "#,
            poll_id
        )
        .fetch_all(pool)
        .await?;

        let ballots = ballot_data
            .into_iter()
            .map(|row| crate::services::rcv::Ballot {
                id: row.id,
                // For anonymous ballots, voter_id is NULL, so use a placeholder UUID
                voter_id: row.voter_id.unwrap_or_else(|| Uuid::nil()),
                rankings: row.candidate_ids.unwrap_or_default(),
            })
            .collect();

        Ok(ballots)
    }
}

impl Voter {
    /// Create a new voter with ballot token
    pub async fn create(
        pool: &PgPool,
        poll_id: Uuid,
        email: Option<String>,
        ip_address: Option<IpNetwork>,
        user_agent: Option<String>,
    ) -> Result<Voter, sqlx::Error> {
        let ballot_token = generate_ballot_token();
        
        let voter_row = sqlx::query!(
            r#"
            INSERT INTO voters (poll_id, email, ballot_token, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, poll_id, email, ballot_token, ip_address, user_agent, 
                      location_data, demographics, invited_at, voted_at
            "#,
            poll_id,
            email,
            ballot_token,
            ip_address,
            user_agent
        )
        .fetch_one(pool)
        .await?;
        
        let voter = Voter {
            id: voter_row.id,
            poll_id: voter_row.poll_id.expect("poll_id cannot be null"),
            email: voter_row.email,
            ballot_token: voter_row.ballot_token,
            ip_address: voter_row.ip_address,
            user_agent: voter_row.user_agent,
            location_data: voter_row.location_data,
            demographics: voter_row.demographics,
            invited_at: voter_row.invited_at.expect("invited_at cannot be null"),
            voted_at: voter_row.voted_at,
        };

        Ok(voter)
    }

    /// Find voter by ballot token
    pub async fn find_by_token(pool: &PgPool, token: &str) -> Result<Option<Voter>, sqlx::Error> {
        let voter_row = sqlx::query!(
            r#"
            SELECT id, poll_id, email, ballot_token, ip_address, user_agent,
                   location_data, demographics, invited_at, voted_at
            FROM voters
            WHERE ballot_token = $1
            "#,
            token
        )
        .fetch_optional(pool)
        .await?;
        
        match voter_row {
            Some(row) => Ok(Some(Voter {
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
            })),
            None => Ok(None),
        }
    }

    /// Mark voter as having voted
    pub async fn mark_as_voted(pool: &PgPool, voter_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE voters SET voted_at = CURRENT_TIMESTAMP WHERE id = $1",
            voter_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if voter has already voted
    pub fn has_voted(&self) -> bool {
        self.voted_at.is_some()
    }
}

/// Generate a cryptographically secure ballot token
fn generate_ballot_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Generate a random string with format: VOTE-YYYY-XXXXXX
    let year = chrono::Utc::now().format("%Y");
    let random_part: String = (0..6)
        .map(|_| {
            let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();
    
    format!("VOTE-{}-{}", year, random_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ballot_token_generation() {
        let token1 = generate_ballot_token();
        let token2 = generate_ballot_token();
        
        assert_ne!(token1, token2);
        assert!(token1.starts_with("VOTE-"));
        assert_eq!(token1.len(), 16); // VOTE-YYYY-XXXXXX = 16 chars
    }

    #[test]
    fn test_voter_has_voted() {
        let mut voter = Voter {
            id: Uuid::new_v4(),
            poll_id: Uuid::new_v4(),
            email: None,
            ballot_token: "test".to_string(),
            ip_address: None,
            user_agent: None,
            location_data: None,
            demographics: None,
            invited_at: Utc::now(),
            voted_at: None,
        };

        assert!(!voter.has_voted());

        voter.voted_at = Some(Utc::now());
        assert!(voter.has_voted());
    }
} 