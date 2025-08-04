use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct EmailService {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Serialize)]
pub struct VoterInvitationRequest {
    #[serde(rename = "pollTitle")]
    pub poll_title: String,
    #[serde(rename = "pollDescription")]
    pub poll_description: Option<String>,
    #[serde(rename = "votingUrl")]
    pub voting_url: String,
    #[serde(rename = "pollOwnerName")]
    pub poll_owner_name: String,
    #[serde(rename = "pollOwnerEmail")]
    pub poll_owner_email: String,
    #[serde(rename = "closesAt")]
    pub closes_at: Option<String>,
    #[serde(rename = "voterName")]
    pub voter_name: Option<String>,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct BulkVoterInvitationRequest {
    #[serde(rename = "pollTitle")]
    pub poll_title: String,
    #[serde(rename = "pollDescription")]
    pub poll_description: Option<String>,
    #[serde(rename = "votingUrl")]
    pub voting_url: String,
    #[serde(rename = "pollOwnerName")]
    pub poll_owner_name: String,
    #[serde(rename = "pollOwnerEmail")]
    pub poll_owner_email: String,
    #[serde(rename = "closesAt")]
    pub closes_at: Option<String>,
    pub recipients: Vec<EmailRecipient>,
}

#[derive(Debug, Serialize)]
pub struct EmailRecipient {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PollResultsRequest {
    #[serde(rename = "pollTitle")]
    pub poll_title: String,
    #[serde(rename = "pollDescription")]
    pub poll_description: Option<String>,
    #[serde(rename = "winnerName")]
    pub winner_name: String,
    #[serde(rename = "totalVotes")]
    pub total_votes: usize,
    #[serde(rename = "resultsUrl")]
    pub results_url: String,
    #[serde(rename = "pollOwnerName")]
    pub poll_owner_name: String,
    #[serde(rename = "voterName")]
    pub voter_name: Option<String>,
    #[serde(rename = "finalRankings")]
    pub final_rankings: Vec<FinalRanking>,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct FinalRanking {
    pub position: usize,
    pub name: String,
    pub votes: f64,
    pub percentage: f64,
}

#[derive(Debug, Deserialize)]
pub struct EmailResponse {
    pub success: bool,
    pub data: Option<EmailResponseData>,
    pub error: Option<EmailError>,
}

#[derive(Debug, Deserialize)]
pub struct EmailResponseData {
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    pub recipient: Option<String>,
    pub sent: Option<usize>,
    pub failed: Option<usize>,
    #[serde(rename = "failedRecipients")]
    pub failed_recipients: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct EmailError {
    pub code: String,
    pub message: String,
}

impl EmailService {
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("EMAIL_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());
        
        let api_key = std::env::var("EMAIL_SERVICE_API_KEY")
            .context("EMAIL_SERVICE_API_KEY environment variable is required")?;

        Ok(Self {
            client: Client::new(),
            base_url,
            api_key,
        })
    }

    pub async fn send_voter_invitation(
        &self,
        request: VoterInvitationRequest,
    ) -> Result<EmailResponse> {
        let url = format!("{}/api/email/voter-invitation", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send HTTP request to email service")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Email service returned error {}: {}", status, text);
        }

        let email_response: EmailResponse = response
            .json()
            .await
            .context("Failed to parse email service response")?;

        Ok(email_response)
    }

    pub async fn send_bulk_voter_invitations(
        &self,
        request: BulkVoterInvitationRequest,
    ) -> Result<EmailResponse> {
        let url = format!("{}/api/email/bulk-voter-invitations", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send HTTP request to email service")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Email service returned error {}: {}", status, text);
        }

        let email_response: EmailResponse = response
            .json()
            .await
            .context("Failed to parse email service response")?;

        Ok(email_response)
    }

    pub async fn send_poll_results(
        &self,
        request: PollResultsRequest,
    ) -> Result<EmailResponse> {
        let url = format!("{}/api/email/poll-results", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send HTTP request to email service")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Email service returned error {}: {}", status, text);
        }

        let email_response: EmailResponse = response
            .json()
            .await
            .context("Failed to parse email service response")?;

        Ok(email_response)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send health check request")?;

        Ok(response.status().is_success())
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new().expect("Failed to create EmailService")
    }
}