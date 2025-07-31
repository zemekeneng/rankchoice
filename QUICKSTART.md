# RankChoice.app - Quick Start Guide

## Prerequisites

- Rust 1.75+ with cargo
- Node.js 18+ and npm
- Docker and Docker Compose
- AWS CLI configured
- Terraform 1.5+

## Local Development Setup

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/rankchoice.git
cd rankchoice
```

### 2. Set Up Environment Variables

```bash
cp .env.example .env
# Edit .env with your local configuration
```

### 3. Start Local Database

```bash
docker-compose up -d postgres
```

### 4. Install Dependencies

#### Backend (Rust)
```bash
cd backend
cargo build
```

#### Frontend (SvelteKit)
```bash
cd frontend
npm install
```

### 5. Run Database Migrations

```bash
cd backend
cargo install sqlx-cli
sqlx migrate run
```

### 6. Start Development Servers

#### Backend API
```bash
cd backend
cargo run
# API will be available at http://localhost:8080
```

#### Frontend
```bash
cd frontend
npm run dev
# Frontend will be available at http://localhost:3000
```

## Project Structure

```
rankchoice/
├── backend/                 # Rust API
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── api/            # API handlers
│   │   ├── models/         # Database models
│   │   ├── services/       # Business logic
│   │   └── rcv/            # RCV algorithm
│   ├── migrations/         # SQL migrations
│   └── Cargo.toml
├── frontend/               # SvelteKit app
│   ├── src/
│   │   ├── lib/           # Shared components & utilities
│   │   │   ├── components/ # Svelte components
│   │   │   ├── stores/    # Svelte stores
│   │   │   └── api/       # API client
│   │   ├── routes/        # SvelteKit routes/pages
│   │   ├── app.html       # App template
│   │   └── app.d.ts       # TypeScript declarations
│   ├── static/            # Static assets
│   └── package.json
├── infrastructure/         # Terraform configs
│   └── terraform/
├── services/              # Supporting services
│   └── email/            # Email service (Node.js)
└── docs/                  # Documentation
```

## Common Development Tasks

### Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
npm test
```

### Adding a New API Endpoint

1. Add route handler in `backend/src/api/`
2. Add model if needed in `backend/src/models/`
3. Add service logic in `backend/src/services/`
4. Update OpenAPI spec
5. Generate frontend types

### Creating a Database Migration

```bash
cd backend
sqlx migrate add <migration_name>
# Edit the migration file
sqlx migrate run
```

### Building for Production

#### Backend
```bash
cd backend
cargo lambda build --release
```

#### Frontend
```bash
cd frontend
npm run build
```

## Deployment

### Deploy to Staging

```bash
cd infrastructure/terraform
terraform workspace select staging
terraform plan
terraform apply
```

### Deploy to Production

```bash
cd infrastructure/terraform
terraform workspace select prod
terraform plan
terraform apply -auto-approve=false
```

## Useful Commands

### Local Lambda Testing
```bash
# Install cargo-lambda
cargo install cargo-lambda

# Run lambda locally
cd backend
cargo lambda watch

# Test with event
cargo lambda invoke --data-file events/test-event.json
```

### Database Access
```bash
# Connect to local database
docker exec -it rankchoice_postgres psql -U rankchoice

# Run SQL file
docker exec -i rankchoice_postgres psql -U rankchoice < script.sql
```

### Log Viewing
```bash
# Backend logs
tail -f backend/logs/app.log

# Frontend logs
cd frontend && npm run logs
```

## Troubleshooting

### Common Issues

1. **Database connection errors**
   - Check if Docker is running
   - Verify DATABASE_URL in .env
   - Run `docker-compose restart postgres`

2. **Rust compilation errors**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean && cargo build`

3. **Frontend build errors**
   - Clear cache: `rm -rf node_modules && npm install`
   - Check Node version: `node --version`

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run

# Frontend debug
npm run dev -- --debug
```

## Contributing

1. Create feature branch: `git checkout -b feature/your-feature`
2. Make changes and test
3. Run linters: `cargo clippy` and `npm run lint`
4. Commit with conventional commits
5. Push and create PR

## Resources

- [Project Planning Document](./PLANNING.md)
- [Makefile Commands Documentation](./docs/MAKEFILE_COMMANDS.md)
- [API Documentation](./docs/API_SPECIFICATION.md)
- [RCV Algorithm Details](./docs/RCV_ALGORITHM.md)
- [Svelte 5 Guide](./docs/SVELTE5_GUIDE.md)
- [Infrastructure Guide](./infrastructure/terraform/README.md)

## Support

- Slack: #rankchoice-dev
- Email: dev@rankchoice.app
- Issues: GitHub Issues 