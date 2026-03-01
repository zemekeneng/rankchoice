use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuthToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AuthToken {
    pub fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        token_type: &str,
        ttl: Duration,
    ) -> Result<AuthToken, sqlx::Error> {
        let token = Self::generate_token();
        let expires_at = Utc::now() + ttl;

        let auth_token = sqlx::query_as::<_, AuthToken>(
            r#"
            INSERT INTO auth_tokens (user_id, token, token_type, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, token, token_type, expires_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(&token)
        .bind(token_type)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        Ok(auth_token)
    }

    pub async fn find_by_token(pool: &PgPool, token: &str) -> Result<Option<AuthToken>, sqlx::Error> {
        let auth_token = sqlx::query_as::<_, AuthToken>(
            "SELECT id, user_id, token, token_type, expires_at, created_at FROM auth_tokens WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(auth_token)
    }

    pub async fn delete_by_token(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM auth_tokens WHERE token = $1")
            .bind(token)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete_for_user(pool: &PgPool, user_id: Uuid, token_type: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM auth_tokens WHERE user_id = $1 AND token_type = $2")
            .bind(user_id)
            .bind(token_type)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM auth_tokens WHERE expires_at < NOW()")
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
