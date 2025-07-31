# RankChoice API Specification

## Overview

The RankChoice API is a RESTful service that powers the ranked-choice voting platform. All endpoints return JSON and follow REST conventions.

## Base URL

- Production: `https://rankchoice.app/api`
- Staging: `https://staging.rankchoice.app/api`

## Authentication

The API uses JWT (JSON Web Tokens) for authentication. Include the token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

## Common Response Format

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "metadata": {
    "timestamp": "2024-01-01T00:00:00Z",
    "version": "1.0",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

## Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    }
  },
  "metadata": { ... }
}
```

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| UNAUTHORIZED | 401 | Missing or invalid authentication |
| FORBIDDEN | 403 | Insufficient permissions |
| NOT_FOUND | 404 | Resource not found |
| VALIDATION_ERROR | 400 | Invalid input data |
| RATE_LIMITED | 429 | Too many requests |
| INTERNAL_ERROR | 500 | Server error |

## Endpoints

### Authentication

#### Register User

```http
POST /auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "name": "John Doe"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "name": "John Doe",
      "role": "pollster",
      "created_at": "2024-01-01T00:00:00Z"
    },
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

#### Login

```http
POST /auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

#### Refresh Token

```http
POST /auth/refresh
Content-Type: application/json

{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### Polls

#### Create Poll

```http
POST /polls
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Best Programming Language 2024",
  "description": "Vote for your favorite programming language",
  "poll_type": "single_winner",
  "opens_at": "2024-01-01T00:00:00Z",
  "closes_at": "2024-12-31T23:59:59Z",
  "is_public": true,
  "registration_required": false,
  "candidates": [
    {
      "name": "Rust",
      "description": "Systems programming language"
    },
    {
      "name": "Python",
      "description": "General-purpose programming language"
    },
    {
      "name": "JavaScript",
      "description": "Web programming language"
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "poll": {
      "id": "650e8400-e29b-41d4-a716-446655440000",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "Best Programming Language 2024",
      "description": "Vote for your favorite programming language",
      "poll_type": "single_winner",
      "num_winners": 1,
      "opens_at": "2024-01-01T00:00:00Z",
      "closes_at": "2024-12-31T23:59:59Z",
      "is_public": true,
      "registration_required": false,
      "created_at": "2024-01-01T00:00:00Z",
      "candidates": [
        {
          "id": "750e8400-e29b-41d4-a716-446655440000",
          "name": "Rust",
          "description": "Systems programming language",
          "display_order": 1
        },
        {
          "id": "850e8400-e29b-41d4-a716-446655440000",
          "name": "Python",
          "description": "General-purpose programming language",
          "display_order": 2
        },
        {
          "id": "950e8400-e29b-41d4-a716-446655440000",
          "name": "JavaScript",
          "description": "Web programming language",
          "display_order": 3
        }
      ]
    }
  }
}
```

#### List User's Polls

```http
GET /polls
Authorization: Bearer <token>

Query Parameters:
- page (int): Page number (default: 1)
- limit (int): Items per page (default: 20, max: 100)
- status (string): Filter by status (active, closed, draft)
- sort (string): Sort field (created_at, title, closes_at)
- order (string): Sort order (asc, desc)
```

#### Get Poll Details

```http
GET /polls/{poll_id}
Authorization: Bearer <token>
```

#### Update Poll

```http
PUT /polls/{poll_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Updated Title",
  "description": "Updated description",
  "closes_at": "2024-12-31T23:59:59Z"
}
```

#### Delete Poll

```http
DELETE /polls/{poll_id}
Authorization: Bearer <token>
```

### Candidates

#### Add Candidate

```http
POST /polls/{poll_id}/candidates
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Go",
  "description": "Concurrent programming language"
}
```

#### Update Candidate

```http
PUT /candidates/{candidate_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Go (Golang)",
  "description": "Updated description"
}
```

#### Reorder Candidates

```http
PUT /polls/{poll_id}/candidates/order
Authorization: Bearer <token>
Content-Type: application/json

{
  "candidate_order": [
    "750e8400-e29b-41d4-a716-446655440000",
    "950e8400-e29b-41d4-a716-446655440000",
    "850e8400-e29b-41d4-a716-446655440000"
  ]
}
```

### Voting

#### Get Ballot

```http
GET /vote/{ballot_token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "poll": {
      "id": "650e8400-e29b-41d4-a716-446655440000",
      "title": "Best Programming Language 2024",
      "description": "Vote for your favorite programming language",
      "poll_type": "single_winner",
      "candidates": [
        {
          "id": "750e8400-e29b-41d4-a716-446655440000",
          "name": "Rust",
          "description": "Systems programming language"
        },
        {
          "id": "850e8400-e29b-41d4-a716-446655440000",
          "name": "Python",
          "description": "General-purpose programming language"
        },
        {
          "id": "950e8400-e29b-41d4-a716-446655440000",
          "name": "JavaScript",
          "description": "Web programming language"
        }
      ]
    },
    "voter": {
      "id": "a50e8400-e29b-41d4-a716-446655440000",
      "has_voted": false
    }
  }
}
```

#### Submit Ballot

```http
POST /vote/{ballot_token}
Content-Type: application/json

{
  "rankings": [
    {
      "candidate_id": "750e8400-e29b-41d4-a716-446655440000",
      "rank": 1
    },
    {
      "candidate_id": "850e8400-e29b-41d4-a716-446655440000",
      "rank": 2
    },
    {
      "candidate_id": "950e8400-e29b-41d4-a716-446655440000",
      "rank": 3
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "ballot": {
      "id": "b50e8400-e29b-41d4-a716-446655440000",
      "submitted_at": "2024-01-01T12:00:00Z"
    },
    "receipt": {
      "receipt_code": "VOTE-2024-A1B2C3",
      "verification_url": "https://rankchoice.app/verify/VOTE-2024-A1B2C3"
    }
  }
}
```

#### Get Voting Receipt

```http
GET /vote/{ballot_token}/receipt
```

### Distribution

#### Send Invitations

```http
POST /polls/{poll_id}/invite
Authorization: Bearer <token>
Content-Type: application/json

{
  "recipients": [
    {
      "email": "voter1@example.com",
      "name": "Voter One"
    },
    {
      "email": "voter2@example.com",
      "name": "Voter Two"
    }
  ],
  "subject": "You're invited to vote!",
  "message": "Please participate in our poll."
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "sent": 2,
    "failed": 0,
    "invitations": [
      {
        "email": "voter1@example.com",
        "status": "sent",
        "ballot_token": "TOKEN-123-ABC"
      },
      {
        "email": "voter2@example.com",
        "status": "sent",
        "ballot_token": "TOKEN-456-DEF"
      }
    ]
  }
}
```

#### Create Registration Link

```http
POST /polls/{poll_id}/registration
Authorization: Bearer <token>
Content-Type: application/json

{
  "max_uses": 100,
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "registration_link": "https://rankchoice.app/register/REG-TOKEN-789",
    "token": "REG-TOKEN-789",
    "max_uses": 100,
    "expires_at": "2024-12-31T23:59:59Z"
  }
}
```

### Results

#### Get Poll Results

```http
GET /polls/{poll_id}/results
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "poll_id": "650e8400-e29b-41d4-a716-446655440000",
    "total_votes": 150,
    "status": "completed",
    "winner": {
      "candidate_id": "750e8400-e29b-41d4-a716-446655440000",
      "name": "Rust",
      "final_votes": 78,
      "percentage": 52.0
    },
    "final_rankings": [
      {
        "position": 1,
        "candidate_id": "750e8400-e29b-41d4-a716-446655440000",
        "name": "Rust",
        "votes": 78,
        "percentage": 52.0
      },
      {
        "position": 2,
        "candidate_id": "850e8400-e29b-41d4-a716-446655440000",
        "name": "Python",
        "votes": 72,
        "percentage": 48.0,
        "eliminated_round": 2
      }
    ]
  }
}
```

#### Get RCV Rounds

```http
GET /polls/{poll_id}/results/rounds
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "rounds": [
      {
        "round_number": 1,
        "vote_counts": {
          "750e8400-e29b-41d4-a716-446655440000": 65,
          "850e8400-e29b-41d4-a716-446655440000": 60,
          "950e8400-e29b-41d4-a716-446655440000": 25
        },
        "eliminated": "950e8400-e29b-41d4-a716-446655440000",
        "exhausted_ballots": 0
      },
      {
        "round_number": 2,
        "vote_counts": {
          "750e8400-e29b-41d4-a716-446655440000": 78,
          "850e8400-e29b-41d4-a716-446655440000": 72
        },
        "winner": "750e8400-e29b-41d4-a716-446655440000",
        "exhausted_ballots": 0
      }
    ]
  }
}
```

### Admin

#### Get Platform Analytics

```http
GET /admin/analytics
Authorization: Bearer <token>

Query Parameters:
- start_date (date): Start date for analytics
- end_date (date): End date for analytics
- granularity (string): day, week, month
```

**Response:**
```json
{
  "success": true,
  "data": {
    "summary": {
      "total_polls": 1250,
      "total_votes": 45000,
      "active_users": 850,
      "revenue": 12500.00
    },
    "geographic_breakdown": {
      "US": {
        "polls": 800,
        "votes": 30000
      },
      "CA": {
        "polls": 200,
        "votes": 8000
      }
    },
    "time_series": [
      {
        "date": "2024-01-01",
        "polls_created": 45,
        "votes_cast": 1200,
        "new_users": 25
      }
    ]
  }
}
```

#### Create Advertisement

```http
POST /admin/ads
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Try our new voting app!",
  "content_url": "https://cdn.rankchoice.app/ads/new-app.jpg",
  "link_url": "https://example.com/promo",
  "target_demographics": {
    "age_range": [18, 35],
    "interests": ["politics", "technology"]
  },
  "target_locations": ["US", "CA"],
  "weight": 2,
  "budget": 1000.00,
  "start_date": "2024-01-01",
  "end_date": "2024-12-31"
}
```

## Rate Limiting

- Anonymous: 100 requests per hour
- Authenticated: 1000 requests per hour
- Voting endpoints: 10 requests per minute

Rate limit headers:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1609459200
```

## Webhooks

Configure webhooks to receive real-time updates:

```http
POST /webhooks
Authorization: Bearer <token>
Content-Type: application/json

{
  "url": "https://example.com/webhook",
  "events": ["poll.created", "poll.closed", "vote.cast"],
  "secret": "webhook_secret_key"
}
```

### Webhook Events

- `poll.created`: New poll created
- `poll.updated`: Poll details updated
- `poll.closed`: Poll closed
- `vote.cast`: Vote submitted
- `results.ready`: Results calculated

### Webhook Payload

```json
{
  "event": "vote.cast",
  "timestamp": "2024-01-01T12:00:00Z",
  "data": {
    "poll_id": "650e8400-e29b-41d4-a716-446655440000",
    "voter_id": "a50e8400-e29b-41d4-a716-446655440000"
  },
  "signature": "sha256=..."
}
```

## SDK Examples

### JavaScript/TypeScript

```typescript
import { RankChoiceClient } from '@rankchoice/sdk';

const client = new RankChoiceClient({
  apiKey: 'your-api-key',
  environment: 'production'
});

// Create a poll
const poll = await client.polls.create({
  title: 'Best Programming Language',
  candidates: [
    { name: 'Rust' },
    { name: 'Python' },
    { name: 'JavaScript' }
  ]
});

// Get results
const results = await client.polls.getResults(poll.id);
```

### Python

```python
from rankchoice import RankChoiceClient

client = RankChoiceClient(
    api_key='your-api-key',
    environment='production'
)

# Create a poll
poll = client.polls.create(
    title='Best Programming Language',
    candidates=[
        {'name': 'Rust'},
        {'name': 'Python'},
        {'name': 'JavaScript'}
    ]
)

# Get results
results = client.polls.get_results(poll.id)
```

### Rust

```rust
use rankchoice_sdk::{RankChoiceClient, CreatePollRequest, Candidate};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RankChoiceClient::new("your-api-key");
    
    // Create a poll
    let poll = client.polls().create(CreatePollRequest {
        title: "Best Programming Language".to_string(),
        candidates: vec![
            Candidate { name: "Rust".to_string(), ..Default::default() },
            Candidate { name: "Python".to_string(), ..Default::default() },
            Candidate { name: "JavaScript".to_string(), ..Default::default() },
        ],
        ..Default::default()
    }).await?;
    
    // Get results
    let results = client.polls().get_results(&poll.id).await?;
    
    Ok(())
}
``` 