# RankChoice.app Email Service

A Node.js microservice for handling email notifications for the RankChoice.app voting platform.

## Features

- 🎯 **Voter Invitations**: Send personalized voting invitations with secure ballot tokens
- 📊 **Poll Results**: Notify participants when voting concludes with detailed results
- 📧 **Bulk Operations**: Efficiently send invitations to multiple voters
- 🔒 **Secure**: API key authentication and CORS protection
- 📈 **Tracking**: Full email delivery tracking and status monitoring
- 🔄 **Retry Logic**: Automatic retry for failed emails with exponential backoff
- 🎨 **Rich Templates**: Beautiful HTML email templates with responsive design

## Quick Start

### 1. Installation

```bash
cd services/email
npm install
```

### 2. Configuration

Copy the environment configuration:
```bash
cp env.example .env
```

Edit `.env` with your SMTP settings:
```env
# Basic Configuration
NODE_ENV=development
PORT=3001
EMAIL_SERVICE_API_KEY=your-secure-api-key

# SMTP Settings
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
FROM_EMAIL=noreply@rankchoice.app
FROM_NAME=RankChoice.app
```

### 3. Development

```bash
# Start in development mode
npm run dev

# Run tests
npm test

# Build for production
npm run build
npm start
```

## API Endpoints

### Health Check
```http
GET /health
GET /health/detailed
```

### Send Voter Invitation
```http
POST /api/email/voter-invitation
X-API-Key: your-api-key
Content-Type: application/json

{
  "pollTitle": "Company Board Election",
  "pollDescription": "Annual election for board members",
  "votingUrl": "https://rankchoice.app/vote/abc123",
  "pollOwnerName": "John Doe",
  "pollOwnerEmail": "john@company.com",
  "closesAt": "2024-12-31T23:59:59Z",
  "voterName": "Jane Smith",
  "to": "jane@company.com"
}
```

### Bulk Voter Invitations
```http
POST /api/email/bulk-voter-invitations
X-API-Key: your-api-key
Content-Type: application/json

{
  "pollTitle": "Company Board Election",
  "votingUrl": "https://rankchoice.app/vote/abc123",
  "pollOwnerName": "John Doe",
  "pollOwnerEmail": "john@company.com",
  "recipients": [
    {"email": "voter1@company.com", "name": "Alice"},
    {"email": "voter2@company.com", "name": "Bob"}
  ]
}
```

### Send Poll Results
```http
POST /api/email/poll-results
X-API-Key: your-api-key
Content-Type: application/json

{
  "pollTitle": "Company Board Election",
  "winnerName": "Alice Johnson",
  "totalVotes": 250,
  "resultsUrl": "https://rankchoice.app/polls/123/results",
  "pollOwnerName": "John Doe",
  "finalRankings": [
    {"position": 1, "name": "Alice Johnson", "votes": 125, "percentage": 50},
    {"position": 2, "name": "Bob Smith", "votes": 125, "percentage": 50}
  ],
  "to": "voter@company.com"
}
```

### Email Tracking
```http
GET /api/email/status/:trackingId
GET /api/email/tracking
POST /api/email/retry-failed
POST /api/email/cleanup
```

## Integration with Backend

Add email sending to your Rust backend by calling the email service:

```rust
// Add to your Cargo.toml
reqwest = { version = "0.11", features = ["json"] }

// Email service client
use reqwest::Client;
use serde_json::json;

async fn send_voter_invitation(
    voter_email: &str,
    poll_title: &str,
    voting_url: &str,
    poll_owner_name: &str,
    poll_owner_email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let payload = json!({
        "pollTitle": poll_title,
        "votingUrl": voting_url,
        "pollOwnerName": poll_owner_name,
        "pollOwnerEmail": poll_owner_email,
        "to": voter_email
    });
    
    let response = client
        .post("http://localhost:3001/api/email/voter-invitation")
        .header("X-API-Key", std::env::var("EMAIL_SERVICE_API_KEY")?)
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        println!("✅ Email sent successfully to {}", voter_email);
    } else {
        println!("❌ Failed to send email: {}", response.status());
    }
    
    Ok(())
}
```

## Email Templates

### Voter Invitation Email
- **Subject**: "You're invited to vote: [Poll Title]"
- **Features**: 
  - Personalized greeting
  - Poll description
  - Clear voting button
  - RCV explanation
  - Deadline information
  - Contact info

### Poll Results Email
- **Subject**: "[Winner] wins '[Poll Title]'"
- **Features**:
  - Winner announcement
  - Final rankings table
  - Vote counts and percentages
  - Link to detailed results
  - Thank you message

## SMTP Providers

### Gmail Setup
1. Enable 2-factor authentication
2. Generate an "App Password" 
3. Use these settings:
```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_SECURE=false
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
```

### AWS SES Setup
```env
SMTP_HOST=email-smtp.us-east-1.amazonaws.com
SMTP_PORT=587
SMTP_USER=your-ses-access-key
SMTP_PASS=your-ses-secret-key
```

### Development Testing
Use [Ethereal Email](https://ethereal.email) for development:
```env
ETHEREAL_USER=your-ethereal-user
ETHEREAL_PASS=your-ethereal-pass
```

## Monitoring

- **Health Checks**: `/health` and `/health/detailed`
- **Email Tracking**: All emails tracked with unique IDs
- **Retry Logic**: Failed emails automatically retried with backoff
- **Cleanup**: Old tracking data automatically cleaned up

## Security

- API key authentication required
- CORS protection
- Rate limiting (configure via reverse proxy)
- Input validation with Zod schemas
- Error sanitization in production

## Testing

```bash
# Run all tests
npm test

# Run tests with coverage
npm test -- --coverage

# Run tests in watch mode
npm test -- --watch
```

## Production Deployment

1. **Build the service**:
```bash
npm run build
```

2. **Set production environment variables**:
```env
NODE_ENV=production
EMAIL_SERVICE_API_KEY=secure-random-key
SMTP_HOST=your-production-smtp
# ... other production settings
```

3. **Start the service**:
```bash
npm start
```

4. **Monitor health**:
```bash
curl http://localhost:3001/health/detailed
```

## License

MIT License - See LICENSE file for details.