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

use crate::models::auth_token::AuthToken;
use crate::models::user::{CreateUserRequest, LoginRequest, User, UserResponse};
use crate::services::email::{EmailService, EmailVerificationRequest, PasswordResetRequest};
use crate::services::ses::SesEmailSender;

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
    frontend_url: Arc<String>,
    email_service: Option<Arc<EmailService>>,
    ses_sender: Option<Arc<SesEmailSender>>,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-256-bit-secret-here-change-in-production".to_string());
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:5174".to_string());

        let email_service = match EmailService::new() {
            Ok(svc) => {
                tracing::info!("Email service configured (EMAIL_SERVICE_URL)");
                Some(Arc::new(svc))
            }
            Err(e) => {
                tracing::warn!("Email service not configured: {}. Verification emails will not be sent.", e);
                None
            }
        };

        Self {
            pool,
            jwt_secret: Arc::new(jwt_secret),
            frontend_url: Arc::new(frontend_url),
            email_service,
            ses_sender: None,
        }
    }

    pub async fn init_ses(&mut self) {
        let use_ses = env::var("USE_SES").unwrap_or_default() == "true";
        if use_ses {
            match SesEmailSender::new().await {
                Ok(sender) => {
                    tracing::info!("SES email sender initialized");
                    self.ses_sender = Some(Arc::new(sender));
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize SES sender: {}", e);
                }
            }
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn register(&self, req: CreateUserRequest) -> Result<AuthResponse, AuthError> {
        let password_hash = self.hash_password(&req.password)?;

        match User::create(&self.pool, req, password_hash).await {
            Ok(user) => {
                // Send verification email (best-effort, don't block registration)
                if let Err(e) = self.send_verification_email(&user).await {
                    tracing::warn!("Failed to send verification email for {}: {}", user.email, e);
                }

                let token = self.generate_token(&user, false)?;
                let refresh_token = self.generate_token(&user, true)?;

                Ok(AuthResponse {
                    user: user.into(),
                    token,
                    refresh_token,
                })
            }
            Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("users_email_key") => {
                Err(AuthError::UserAlreadyExists)
            }
            Err(e) => {
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

    pub async fn verify_email(&self, token: &str) -> Result<(), AuthError> {
        let auth_token = AuthToken::find_by_token(&self.pool, token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if auth_token.token_type != "email_verification" {
            return Err(AuthError::InvalidToken);
        }

        if auth_token.is_expired() {
            AuthToken::delete_by_token(&self.pool, token).await?;
            return Err(AuthError::TokenExpired);
        }

        User::set_email_verified(&self.pool, auth_token.user_id).await?;
        AuthToken::delete_by_token(&self.pool, token).await?;

        Ok(())
    }

    pub async fn forgot_password(&self, email: &str) -> Result<(), AuthError> {
        // Always return Ok to prevent email enumeration
        let user = match User::find_by_email(&self.pool, email).await? {
            Some(u) => u,
            None => return Ok(()),
        };

        AuthToken::delete_for_user(&self.pool, user.id, "password_reset").await?;

        let auth_token = AuthToken::create(
            &self.pool,
            user.id,
            "password_reset",
            Duration::hours(1),
        )
        .await?;

        let reset_url = format!("{}/reset-password?token={}", self.frontend_url, auth_token.token);

        if let Some(ref ses) = self.ses_sender {
            if let Err(e) = ses
                .send_password_reset_email(
                    &user.email,
                    user.name.as_deref(),
                    &reset_url,
                    "1 hour",
                )
                .await
            {
                tracing::error!("SES: Failed to send password reset email to {}: {}", user.email, e);
            }
        } else if let Some(ref email_svc) = self.email_service {
            let request = PasswordResetRequest {
                reset_url,
                user_name: user.name.clone(),
                expires_in: "1 hour".to_string(),
                to: user.email.clone(),
            };
            if let Err(e) = email_svc.send_password_reset(request).await {
                tracing::error!("Failed to send password reset email to {}: {}", user.email, e);
            }
        } else {
            tracing::warn!("No email sender configured, skipping password reset email for {}", user.email);
        }

        Ok(())
    }

    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AuthError> {
        let auth_token = AuthToken::find_by_token(&self.pool, token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if auth_token.token_type != "password_reset" {
            return Err(AuthError::InvalidToken);
        }

        if auth_token.is_expired() {
            AuthToken::delete_by_token(&self.pool, token).await?;
            return Err(AuthError::TokenExpired);
        }

        let password_hash = self.hash_password(new_password)?;
        User::update_password(&self.pool, auth_token.user_id, &password_hash).await?;
        AuthToken::delete_by_token(&self.pool, token).await?;

        Ok(())
    }

    pub async fn resend_verification(&self, user_id: Uuid) -> Result<(), AuthError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if user.email_verified {
            return Ok(());
        }

        self.send_verification_email(&user).await
    }

    async fn send_verification_email(&self, user: &User) -> Result<(), AuthError> {
        AuthToken::delete_for_user(&self.pool, user.id, "email_verification").await?;

        let auth_token = AuthToken::create(
            &self.pool,
            user.id,
            "email_verification",
            Duration::hours(24),
        )
        .await?;

        let verification_url = format!(
            "{}/verify-email?token={}",
            self.frontend_url, auth_token.token
        );

        if let Some(ref ses) = self.ses_sender {
            if let Err(e) = ses
                .send_verification_email(
                    &user.email,
                    user.name.as_deref(),
                    &verification_url,
                )
                .await
            {
                tracing::error!("SES: Failed to send verification email to {}: {}", user.email, e);
            } else {
                tracing::info!("Verification email sent via SES to {}", user.email);
            }
        } else if let Some(ref email_svc) = self.email_service {
            let request = EmailVerificationRequest {
                verification_url,
                user_name: user.name.clone(),
                to: user.email.clone(),
            };
            match email_svc.send_email_verification(request).await {
                Ok(_) => tracing::info!("Verification email sent to {} (via email service)", user.email),
                Err(e) => tracing::error!(
                    "Failed to send verification email to {}: {}. Is the email service running on EMAIL_SERVICE_URL?",
                    user.email, e
                ),
            }
        } else {
            tracing::warn!(
                "No email sender configured (set EMAIL_SERVICE_API_KEY or USE_SES=true). Skipping verification email for {}",
                user.email
            );
        }

        Ok(())
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