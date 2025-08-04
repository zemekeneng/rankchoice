use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{env, sync::Arc};
use uuid::Uuid;

use crate::models::user::{CreateUserRequest, LoginRequest, User, UserResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub email: String,
    pub role: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Password hashing error")]
    PasswordHash,
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
}

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
    jwt_secret: Arc<String>,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-256-bit-secret-here-change-in-production".to_string());
        
        Self { 
            pool, 
            jwt_secret: Arc::new(jwt_secret) 
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn register(&self, req: CreateUserRequest) -> Result<AuthResponse, AuthError> {
        // Hash the password
        let password_hash = self.hash_password(&req.password)?;

        // Create the user directly - let database constraint handle duplicates atomically
        match User::create(&self.pool, req, password_hash).await {
            Ok(user) => {
                // Generate tokens
                let token = self.generate_token(&user, false)?;
                let refresh_token = self.generate_token(&user, true)?;

                Ok(AuthResponse {
                    user: user.into(),
                    token,
                    refresh_token,
                })
            }
            Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("users_email_key") => {
                // Database constraint violation = user already exists
                Err(AuthError::UserAlreadyExists)
            }
            Err(e) => {
                // Other database errors
                Err(AuthError::Database(e))
            }
        }
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Find user by email
        let user = User::find_by_email(&self.pool, &req.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !self.verify_password(&req.password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate tokens
        let token = self.generate_token(&user, false)?;
        let refresh_token = self.generate_token(&user, true)?;

        Ok(AuthResponse {
            user: user.into(),
            token,
            refresh_token,
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<String, AuthError> {
        let claims = self.verify_token(refresh_token)?;
        
        // Find user to generate new token
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;
        
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        // Generate new access token
        self.generate_token(&user, false)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        let token_data: TokenData<Claims> = decode(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    pub fn generate_token(&self, user: &User, is_refresh: bool) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp_duration = if is_refresh {
            Duration::days(7) // Refresh token expires in 7 days
        } else {
            Duration::hours(24) // Access token expires in 24 hours
        };

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: (now + exp_duration).timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::PasswordHash)?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AuthError::PasswordHash)?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
} 