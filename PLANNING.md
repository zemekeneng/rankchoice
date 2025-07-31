# RankChoice.app - Project Planning Document

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Project Overview](#project-overview)
3. [User Stories & Requirements](#user-stories--requirements)
4. [Technical Architecture](#technical-architecture)
5. [Database Design](#database-design)
6. [API Design](#api-design)
7. [Frontend Design](#frontend-design)
8. [Security Considerations](#security-considerations)
9. [Infrastructure & Deployment](#infrastructure--deployment)
10. [Development Roadmap](#development-roadmap)
11. [Technical Decisions](#technical-decisions)

## Executive Summary

RankChoice.app is a modern web application for creating and managing ranked-choice voting polls. The system enables pollsters to create elections, distribute unique voting links, collect ranked ballots, and visualize results using the ranked-choice voting methodology. Built with Rust for performance and deployed on AWS serverless infrastructure for scalability.

### Key Features
- Create ranked-choice voting polls with multiple candidates
- Distribute unique voting links via email (extensible to SMS, social media)
- Voter registration system
- Intuitive drag-and-drop ranking interface
- Real-time vote visualization
- Advertising integration based on voter demographics
- Comprehensive analytics dashboard

## Project Overview

### What is Ranked-Choice Voting?
Ranked-choice voting (RCV) is an electoral system where voters rank candidates in order of preference. Winners are determined through instant runoff rounds:
- **Single-winner**: Candidate needs >50% of votes to win
- **Multi-winner**: Candidates need >25% of votes (for 3-winner elections)

### Project Goals
1. Provide an easy-to-use platform for creating RCV polls
2. Ensure secure, anonymous voting with unique ballot links
3. Deliver real-time, transparent results visualization
4. Scale efficiently using serverless architecture
5. Monetize through targeted advertising post-vote

## User Stories & Requirements

### Pollster Stories
1. **Create Poll**
   - As a pollster, I can create a poll with a title, description, and multiple candidates
   - I can set poll open/close dates
   - I can choose between single-winner or multi-winner RCV
   - I can preview the ballot before publishing

2. **Distribute Poll**
   - As a pollster, I can generate unique voting links for each voter
   - I can send invitations via email with personalized links
   - I can generate a registration link for open enrollment
   - I can track invitation delivery status

3. **Manage Poll**
   - As a pollster, I can view real-time results
   - I can export results in various formats (CSV, PDF)
   - I can close voting early if needed
   - I can view voter participation metrics

### Voter Stories
1. **Vote**
   - As a voter with a ballot link, I can view all candidates
   - I can rank candidates using drag-and-drop
   - I can skip rankings (partial ballot)
   - I can review and submit my ballot

2. **View Results**
   - As a voter, after submitting, I see how my vote is counted
   - I see a graphical explanation of the RCV rounds
   - I can share results on social media

3. **Registration**
   - As a voter, I can register for a poll using a registration link
   - I provide minimal information (email, optional demographics)
   - I receive a unique voting link via email

### Admin Stories
1. **Advertising Management**
   - As an admin, I can upload advertising content
   - I can target ads based on location/demographics
   - I can set ad display rules and frequency

2. **Analytics**
   - As an admin, I can view platform-wide metrics
   - I see geographic breakdown of polls/votes
   - I can track ad performance and revenue

## Technical Architecture

### High-Level Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                        CloudFront CDN                        │
│                   (Static Assets & API Gateway)              │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                      API Gateway (REST)                      │
└─────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┴───────────────────────┐
        ▼                                               ▼
┌─────────────────┐                           ┌─────────────────┐
│   Lambda Functions (Rust)                   │  Lambda Functions │
│   - Poll CRUD                              │  (Node.js)        │
│   - Voting Logic                           │  - Email Service  │
│   - RCV Tabulation                         │  - File Upload    │
└─────────────────┘                           └─────────────────┘
        │                                               │
        └───────────────────┬───────────────────────────┘
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Aurora Serverless v2                      │
│                      (PostgreSQL)                            │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                         S3 Buckets                           │
│           - Static Assets                                    │
│           - Ad Content                                       │
│           - Export Files                                     │
└─────────────────────────────────────────────────────────────┘
```

### Technology Stack

#### Backend
- **Core API**: Rust (using AWS Lambda Runtime)
  - Framework: Axum or Actix-web (Lambda compatible)
  - ORM: Diesel or SQLx for PostgreSQL
  - Serialization: Serde
- **Supporting Services**: Node.js/TypeScript
  - Email sending (AWS SES integration)
  - File processing
- **Database**: Aurora Serverless v2 PostgreSQL
- **Caching**: DynamoDB for session data
- **Queue**: SQS for async tasks

#### Frontend
- **Framework**: SvelteKit with TypeScript
- **UI Library**: Tailwind CSS + Skeleton UI or Melt UI
- **State Management**: Svelte stores (built-in)
- **Drag & Drop**: svelte-dnd-action or custom with HTML5 DnD
- **Data Viz**: Layer Cake (Svelte) or D3.js
- **Build Tool**: Vite (built into SvelteKit)

#### Infrastructure
- **CDN**: CloudFront
- **Compute**: Lambda Functions
- **API**: API Gateway
- **Database**: Aurora Serverless v2
- **Storage**: S3
- **Email**: SES
- **IaC**: Terraform
- **CI/CD**: GitHub Actions

## Database Design

### Core Tables

```sql
-- Users table (pollsters and admins)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'pollster',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Polls table
CREATE TABLE polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    title VARCHAR(500) NOT NULL,
    description TEXT,
    poll_type VARCHAR(50) NOT NULL DEFAULT 'single_winner',
    num_winners INTEGER DEFAULT 1,
    opens_at TIMESTAMP WITH TIME ZONE,
    closes_at TIMESTAMP WITH TIME ZONE,
    is_public BOOLEAN DEFAULT false,
    registration_required BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Candidates/Choices table
CREATE TABLE candidates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID REFERENCES polls(id) ON DELETE CASCADE,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    display_order INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Voters table
CREATE TABLE voters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID REFERENCES polls(id) ON DELETE CASCADE,
    email VARCHAR(255),
    ballot_token VARCHAR(255) UNIQUE NOT NULL,
    ip_address INET,
    user_agent TEXT,
    location_data JSONB,
    demographics JSONB,
    invited_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    voted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(poll_id, email)
);

-- Ballots table
CREATE TABLE ballots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voter_id UUID REFERENCES voters(id) ON DELETE CASCADE,
    poll_id UUID REFERENCES polls(id) ON DELETE CASCADE,
    submitted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    ip_address INET,
    UNIQUE(voter_id)
);

-- Rankings table
CREATE TABLE rankings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ballot_id UUID REFERENCES ballots(id) ON DELETE CASCADE,
    candidate_id UUID REFERENCES candidates(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL,
    UNIQUE(ballot_id, candidate_id),
    UNIQUE(ballot_id, rank)
);

-- Advertisements table
CREATE TABLE advertisements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    content_url TEXT NOT NULL,
    target_demographics JSONB,
    target_locations JSONB,
    weight INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Ad impressions table
CREATE TABLE ad_impressions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ad_id UUID REFERENCES advertisements(id),
    voter_id UUID REFERENCES voters(id),
    displayed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    clicked BOOLEAN DEFAULT false
);

-- Create indexes
CREATE INDEX idx_polls_user_id ON polls(user_id);
CREATE INDEX idx_candidates_poll_id ON candidates(poll_id);
CREATE INDEX idx_voters_poll_id ON voters(poll_id);
CREATE INDEX idx_ballots_poll_id ON ballots(poll_id);
CREATE INDEX idx_rankings_ballot_id ON rankings(ballot_id);
CREATE INDEX idx_ad_impressions_ad_id ON ad_impressions(ad_id);
```

## API Design

### REST API Endpoints

#### Authentication
```
POST   /api/auth/register     - Register new user
POST   /api/auth/login        - Login user
POST   /api/auth/logout       - Logout user
POST   /api/auth/refresh      - Refresh token
```

#### Polls
```
GET    /api/polls             - List user's polls
POST   /api/polls             - Create new poll
GET    /api/polls/:id         - Get poll details
PUT    /api/polls/:id         - Update poll
DELETE /api/polls/:id         - Delete poll
POST   /api/polls/:id/close   - Close poll early
```

#### Candidates
```
GET    /api/polls/:id/candidates     - List candidates
POST   /api/polls/:id/candidates     - Add candidate
PUT    /api/candidates/:id           - Update candidate
DELETE /api/candidates/:id           - Delete candidate
PUT    /api/polls/:id/candidates/order - Reorder candidates
```

#### Voting
```
GET    /api/vote/:token       - Get ballot by token
POST   /api/vote/:token       - Submit ballot
GET    /api/vote/:token/receipt - Get voting receipt
```

#### Distribution
```
POST   /api/polls/:id/invite         - Send invitations
GET    /api/polls/:id/voters         - List voters
POST   /api/polls/:id/registration   - Create registration link
POST   /api/register/:token          - Register to vote
```

#### Results
```
GET    /api/polls/:id/results        - Get results
GET    /api/polls/:id/results/rounds - Get RCV rounds
GET    /api/polls/:id/results/export - Export results
```

#### Admin
```
GET    /api/admin/analytics          - Platform analytics
POST   /api/admin/ads                - Create advertisement
GET    /api/admin/ads                - List advertisements
PUT    /api/admin/ads/:id            - Update advertisement
DELETE /api/admin/ads/:id            - Delete advertisement
```

### API Response Format
```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "metadata": {
    "timestamp": "2024-01-01T00:00:00Z",
    "version": "1.0"
  }
}
```

## Frontend Design

### Key Pages

1. **Landing Page** (`/`)
   - Hero section explaining RCV
   - Features overview
   - CTA to create poll or learn more

2. **Dashboard** (`/dashboard`)
   - List of user's polls
   - Quick stats (active polls, total votes)
   - Create new poll button

3. **Create/Edit Poll** (`/polls/new`, `/polls/:id/edit`)
   - Poll details form
   - Candidate management (add/remove/reorder)
   - Settings (dates, visibility, registration)
   - Preview mode

4. **Poll Management** (`/polls/:id`)
   - Real-time vote count
   - Voter list and status
   - Distribution tools
   - Results visualization

5. **Voting Page** (`/vote/:token`)
   - Candidate cards with drag-and-drop
   - Ranking visualization
   - Submit confirmation
   - Mobile-optimized

6. **Results Page** (`/polls/:id/results`)
   - Winner announcement
   - Round-by-round visualization
   - Individual ballot tracking
   - Share buttons

7. **Admin Dashboard** (`/admin`)
   - Platform metrics
   - Ad management
   - User management

### UI/UX Considerations
- Mobile-first design
- Accessibility (WCAG 2.1 AA)
- Progressive enhancement
- Offline capability for voting
- Real-time updates using WebSockets

## Security Considerations

### Authentication & Authorization
- JWT tokens with refresh mechanism
- Role-based access control (RBAC)
- Secure password storage (Argon2)
- Rate limiting on auth endpoints

### Voting Security
- Unique, cryptographically secure ballot tokens
- One-time use tokens
- IP-based fraud detection
- Optional voter registration/verification
- Audit trail for all votes

### Data Protection
- Encryption at rest (Aurora)
- Encryption in transit (TLS 1.3)
- PII data minimization
- GDPR compliance features
- Regular security audits

### Infrastructure Security
- VPC isolation
- Security groups and NACLs
- AWS WAF for DDoS protection
- CloudTrail for audit logging
- Secrets Manager for credentials

## Infrastructure & Deployment

### AWS Resources (Terraform)

```hcl
# Main infrastructure components
- VPC with public/private subnets
- Aurora Serverless v2 cluster
- Lambda functions (Rust & Node.js)
- API Gateway REST API
- CloudFront distribution
- S3 buckets (static, uploads)
- SES for email
- Route53 for DNS
- ACM for SSL certificates
- CloudWatch for monitoring
- X-Ray for tracing
```

### Deployment Pipeline
1. **Development**
   - Local development with LocalStack
   - Unit tests (90% coverage target)
   - Integration tests

2. **Staging**
   - Deploy to staging environment
   - Run E2E tests
   - Performance testing

3. **Production**
   - Blue-green deployment
   - Canary releases for Lambda
   - Automated rollback on errors

### Monitoring & Observability
- CloudWatch metrics and alarms
- X-Ray distributed tracing
- Custom dashboards
- Error tracking (Sentry)
- Uptime monitoring

## Development Roadmap

### Phase 1: MVP (Weeks 1-6)
- [ ] Basic user authentication
- [ ] Create single-winner polls
- [ ] Add candidates
- [ ] Generate voting links
- [ ] Basic voting interface
- [ ] Simple results display
- [ ] Deploy to AWS

### Phase 2: Core Features (Weeks 7-10)
- [ ] Email distribution
- [ ] Drag-and-drop ranking
- [ ] RCV tabulation engine
- [ ] Results visualization
- [ ] Voter registration system
- [ ] Multi-winner polls

### Phase 3: Enhanced Features (Weeks 11-14)
- [ ] Real-time updates
- [ ] Advanced analytics
- [ ] Export functionality
- [ ] Progressive Web App (PWA) with offline support
- [ ] Social media integration

### Phase 4: Monetization (Weeks 15-16)
- [ ] Advertisement system
- [ ] Geographic targeting
- [ ] Ad performance tracking
- [ ] Billing integration

### Phase 5: Scale & Optimize (Ongoing)
- [ ] Performance optimization
- [ ] Cost optimization
- [ ] Additional distribution channels
- [ ] API for third-party integration
- [ ] White-label options

## Technical Decisions

### Why Rust for Lambda?
- Performance: Fast cold starts and execution
- Memory safety: Fewer runtime errors
- Cost efficiency: Lower memory usage
- Type safety: Catch errors at compile time

### Why Aurora Serverless?
- Auto-scaling capability
- Pay-per-use pricing
- PostgreSQL compatibility
- Managed backups and maintenance

### Why Terraform?
- Infrastructure as Code
- Version control for infrastructure
- Multi-environment management
- AWS-native support

### Trade-offs
- **Complexity**: Rust has steeper learning curve
- **Cold starts**: Mitigated by Lambda SnapStart
- **Vendor lock-in**: AWS-specific services
- **Cost**: Serverless can be expensive at scale

## Success Metrics
- Page load time < 2 seconds
- API response time < 200ms (p95)
- 99.9% uptime
- < $0.01 cost per vote
- Zero data breaches
- User satisfaction > 4.5/5

## Next Steps
1. Set up development environment
2. Create GitHub repository
3. Initialize Terraform configuration
4. Implement authentication service
5. Build poll creation API
6. Develop voting interface prototype 