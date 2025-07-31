use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Candidate {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCandidateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCandidateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderCandidatesRequest {
    pub candidate_order: Vec<Uuid>,
}

impl Candidate {
    pub async fn find_by_poll_id(pool: &PgPool, poll_id: Uuid) -> Result<Vec<Candidate>, sqlx::Error> {
        let candidates = sqlx::query_as::<_, Candidate>(
            "SELECT id, poll_id, name, description, display_order, created_at FROM candidates WHERE poll_id = $1 ORDER BY display_order ASC"
        )
        .bind(poll_id)
        .fetch_all(pool)
        .await?;

        Ok(candidates)
    }

    pub async fn find_by_id(pool: &PgPool, candidate_id: Uuid) -> Result<Option<Candidate>, sqlx::Error> {
        let candidate = sqlx::query_as::<_, Candidate>(
            "SELECT id, poll_id, name, description, display_order, created_at FROM candidates WHERE id = $1"
        )
        .bind(candidate_id)
        .fetch_optional(pool)
        .await?;

        Ok(candidate)
    }

    pub async fn create(
        pool: &PgPool,
        poll_id: Uuid,
        req: CreateCandidateRequest,
    ) -> Result<Candidate, sqlx::Error> {
        // Get the next display order
        let next_order: (Option<i32>,) = sqlx::query_as(
            "SELECT MAX(display_order) FROM candidates WHERE poll_id = $1"
        )
        .bind(poll_id)
        .fetch_one(pool)
        .await?;

        let display_order = next_order.0.unwrap_or(0) + 1;

        let candidate = sqlx::query_as::<_, Candidate>(
            r#"
            INSERT INTO candidates (poll_id, name, description, display_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id, poll_id, name, description, display_order, created_at
            "#,
        )
        .bind(poll_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(display_order)
        .fetch_one(pool)
        .await?;

        Ok(candidate)
    }

    pub async fn update(
        pool: &PgPool,
        candidate_id: Uuid,
        req: UpdateCandidateRequest,
    ) -> Result<Option<Candidate>, sqlx::Error> {
        // Simple approach: construct update based on what fields are provided
        if req.name.is_some() && req.description.is_some() {
            let candidate = sqlx::query_as::<_, Candidate>(
                "UPDATE candidates SET name = $1, description = $2 WHERE id = $3 RETURNING id, poll_id, name, description, display_order, created_at"
            )
            .bind(&req.name)
            .bind(&req.description)
            .bind(candidate_id)
            .fetch_optional(pool)
            .await?;
            Ok(candidate)
        } else if req.name.is_some() {
            let candidate = sqlx::query_as::<_, Candidate>(
                "UPDATE candidates SET name = $1 WHERE id = $2 RETURNING id, poll_id, name, description, display_order, created_at"
            )
            .bind(&req.name)
            .bind(candidate_id)
            .fetch_optional(pool)
            .await?;
            Ok(candidate)
        } else if req.description.is_some() {
            let candidate = sqlx::query_as::<_, Candidate>(
                "UPDATE candidates SET description = $1 WHERE id = $2 RETURNING id, poll_id, name, description, display_order, created_at"
            )
            .bind(&req.description)
            .bind(candidate_id)
            .fetch_optional(pool)
            .await?;
            Ok(candidate)
        } else {
            // No changes requested, return current candidate
            Self::find_by_id(pool, candidate_id).await
        }
    }

    pub async fn delete(pool: &PgPool, candidate_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM candidates WHERE id = $1")
            .bind(candidate_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn reorder(
        pool: &PgPool,
        poll_id: Uuid,
        candidate_order: Vec<Uuid>,
    ) -> Result<Vec<Candidate>, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Update display order for each candidate
        for (index, candidate_id) in candidate_order.iter().enumerate() {
            sqlx::query(
                "UPDATE candidates SET display_order = $1 WHERE id = $2 AND poll_id = $3"
            )
            .bind(index as i32 + 1)
            .bind(candidate_id)
            .bind(poll_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Return updated candidates
        Self::find_by_poll_id(pool, poll_id).await
    }
} 