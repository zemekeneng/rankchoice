use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::candidate::{Candidate, CreateCandidateRequest};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Poll {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub poll_type: String,
    pub num_winners: i32,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_public: bool,
    pub registration_required: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub title: String,
    pub description: Option<String>,
    pub poll_type: Option<String>,
    pub num_winners: Option<i32>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_public: Option<bool>,
    pub registration_required: Option<bool>,
    pub candidates: Vec<CreateCandidateRequest>,
}



#[derive(Debug, Deserialize)]
pub struct UpdatePollRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_public: Option<bool>,
    pub registration_required: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PollResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub poll_type: String,
    pub num_winners: i32,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_public: bool,
    pub registration_required: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct PollListItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub poll_type: String,
    pub num_winners: i32,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub candidate_count: i64,
    pub vote_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct PollListQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub status: Option<String>, // active, closed, draft
    pub sort: Option<String>,   // created_at, title, closes_at
    pub order: Option<String>,  // asc, desc
}

impl Poll {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreatePollRequest,
    ) -> Result<PollResponse, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Create the poll
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            INSERT INTO polls (user_id, title, description, poll_type, num_winners, opens_at, closes_at, is_public, registration_required)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, title, description, poll_type, num_winners, opens_at, closes_at, is_public, registration_required, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.poll_type.unwrap_or_else(|| "single_winner".to_string()))
        .bind(req.num_winners.unwrap_or(1))
        .bind(req.opens_at)
        .bind(req.closes_at)
        .bind(req.is_public.unwrap_or(false))
        .bind(req.registration_required.unwrap_or(false))
        .fetch_one(&mut *tx)
        .await?;

        // Create candidates
        let mut candidates = Vec::new();
        for (index, candidate_req) in req.candidates.iter().enumerate() {
            let candidate = sqlx::query_as::<_, Candidate>(
                r#"
                INSERT INTO candidates (poll_id, name, description, display_order)
                VALUES ($1, $2, $3, $4)
                RETURNING id, poll_id, name, description, display_order, created_at
                "#,
            )
            .bind(poll.id)
            .bind(&candidate_req.name)
            .bind(&candidate_req.description)
            .bind(index as i32 + 1)
            .fetch_one(&mut *tx)
            .await?;

            candidates.push(candidate);
        }

        tx.commit().await?;

        Ok(PollResponse {
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
        })
    }

    pub async fn find_by_id_and_user(
        pool: &PgPool,
        poll_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<PollResponse>, sqlx::Error> {
        let poll = sqlx::query_as::<_, Poll>(
            "SELECT id, user_id, title, description, poll_type, num_winners, opens_at, closes_at, is_public, registration_required, created_at, updated_at FROM polls WHERE id = $1 AND user_id = $2"
        )
        .bind(poll_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some(poll) = poll {
            let candidates = Candidate::find_by_poll_id(pool, poll.id).await?;
            
            Ok(Some(PollResponse {
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
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn find_by_id(pool: &PgPool, poll_id: Uuid) -> Result<Option<PollResponse>, sqlx::Error> {
        let poll = sqlx::query_as::<_, Poll>(
            "SELECT id, user_id, title, description, poll_type, num_winners, opens_at, closes_at, is_public, registration_required, created_at, updated_at FROM polls WHERE id = $1"
        )
        .bind(poll_id)
        .fetch_optional(pool)
        .await?;

        if let Some(poll) = poll {
            let candidates = Candidate::find_by_poll_id(pool, poll.id).await?;
            
            Ok(Some(PollResponse {
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
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
        query: &PollListQuery,
    ) -> Result<(Vec<PollListItem>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = (page - 1) * limit;

        let mut where_clauses = vec!["p.user_id = $1".to_string()];

        // Add status filter
        if let Some(status) = &query.status {
            match status.as_str() {
                "active" => {
                    where_clauses.push(format!("(p.opens_at IS NULL OR p.opens_at <= NOW()) AND (p.closes_at IS NULL OR p.closes_at > NOW())"));
                }
                "closed" => {
                    where_clauses.push(format!("p.closes_at IS NOT NULL AND p.closes_at <= NOW()"));
                }
                "draft" => {
                    where_clauses.push(format!("p.opens_at IS NOT NULL AND p.opens_at > NOW()"));
                }
                _ => {} // Invalid status, ignore
            }
        }

        let where_clause = where_clauses.join(" AND ");

        // Build ORDER BY clause
        let sort_field = match query.sort.as_deref() {
            Some("title") => "p.title",
            Some("closes_at") => "p.closes_at",
            _ => "p.created_at", // default
        };
        let order = match query.order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC", // default
        };

        let query_sql = format!(
            r#"
            SELECT 
                p.id,
                p.title,
                p.description,
                p.poll_type,
                p.num_winners,
                p.opens_at,
                p.closes_at,
                p.is_public,
                p.created_at,
                COUNT(DISTINCT c.id) as candidate_count,
                COUNT(DISTINCT b.id) as vote_count
            FROM polls p
            LEFT JOIN candidates c ON p.id = c.poll_id
            LEFT JOIN ballots b ON p.id = b.poll_id
            WHERE {}
            GROUP BY p.id, p.title, p.description, p.poll_type, p.num_winners, p.opens_at, p.closes_at, p.is_public, p.created_at
            ORDER BY {} {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, sort_field, order, limit, offset
        );

        let polls = sqlx::query_as::<_, PollListItem>(&query_sql)
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM polls p WHERE {}",
            where_clause
        );
        let total_count: (i64,) = sqlx::query_as(&count_query)
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        Ok((polls, total_count.0))
    }

    pub async fn update(
        pool: &PgPool,
        poll_id: Uuid,
        user_id: Uuid,
        req: UpdatePollRequest,
    ) -> Result<Option<PollResponse>, sqlx::Error> {
        // Get the current poll first
        let current_poll = sqlx::query_as::<_, Poll>(
            "SELECT id, user_id, title, description, poll_type, num_winners, opens_at, closes_at, is_public, registration_required, created_at, updated_at FROM polls WHERE id = $1 AND user_id = $2"
        )
        .bind(poll_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let current_poll = match current_poll {
            Some(poll) => poll,
            None => return Ok(None),
        };

        // Use current values as defaults for fields not being updated
        let title = req.title.unwrap_or(current_poll.title);
        let description = req.description.or(current_poll.description);
        let opens_at = req.opens_at.or(current_poll.opens_at);
        let closes_at = req.closes_at.or(current_poll.closes_at);
        let is_public = req.is_public.unwrap_or(current_poll.is_public);
        let registration_required = req.registration_required.unwrap_or(current_poll.registration_required);

        // Update the poll
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            UPDATE polls 
            SET title = $1, description = $2, opens_at = $3, closes_at = $4, 
                is_public = $5, registration_required = $6, updated_at = CURRENT_TIMESTAMP
            WHERE id = $7 AND user_id = $8
            RETURNING id, user_id, title, description, poll_type, num_winners, opens_at, closes_at, is_public, registration_required, created_at, updated_at
            "#,
        )
        .bind(title)
        .bind(description)
        .bind(opens_at)
        .bind(closes_at)
        .bind(is_public)
        .bind(registration_required)
        .bind(poll_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let candidates = Candidate::find_by_poll_id(pool, poll.id).await?;
        
        Ok(Some(PollResponse {
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
        }))
    }

    pub async fn delete(pool: &PgPool, poll_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM polls WHERE id = $1 AND user_id = $2")
            .bind(poll_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
} 