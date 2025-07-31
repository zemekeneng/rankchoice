# RankChoice.app - Makefile Commands Documentation

This document provides comprehensive documentation for all available Makefile commands in the RankChoice.app development environment.

## Table of Contents

- [Quick Reference](#quick-reference)
- [Development Environment](#development-environment)
- [Individual Services](#individual-services)
- [Installation & Setup](#installation--setup)
- [Database Management](#database-management)
- [Building & Testing](#building--testing)
- [Utilities & Maintenance](#utilities--maintenance)
- [Monitoring & Debugging](#monitoring--debugging)
- [Production & Deployment](#production--deployment)
- [Common Workflows](#common-workflows)

## Quick Reference

| Command | Description |
|---------|-------------|
| `make help` | Show all available commands |
| `make quick-start` | Install dependencies and start all services |
| `make dev-bg` | Start full development environment in background |
| `make stop` | Stop all development services |
| `make smart-restart` | Smart restart - no recompilation (fastest for port conflicts) |
| `make fast-restart` | Fast restart - only recompiles app code (recommended) |
| `make force-restart` | Force restart with full clean rebuild (slowest) |
| `make kill-ports` | Kill processes using development ports (8080, 5173) |
| `make status` | Show status of all services |
| `make health` | Check health of all services |
| `make logs` | Show logs from all services |
| `make restart` | Restart all services |

## Development Environment

### `make dev`
**Start the full development environment (blocking)**

Starts all services in the correct order and displays output in the terminal. This is a blocking command, so you'll see logs from all services but can't use the terminal for other commands.

```bash
make dev
```

**What it does:**
1. Checks dependencies
2. Starts Docker services (database, localstack, mailhog)
3. Waits for database to be ready
4. Starts backend and frontend in parallel
5. Shows output from all services

**Use when:** You want to see live output from all services during development.

### `make dev-bg`
**Start the full development environment in background**

Starts all services in background mode, freeing up your terminal for other commands.

```bash
make dev-bg
```

**What it does:**
1. Checks dependencies
2. Starts Docker services
3. Starts backend in background (logs to `logs/backend.log`)
4. Starts frontend in background (logs to `logs/frontend.log`)
5. Shows status of all services

**Use when:** You want to start everything and continue working in the terminal.

### `make stop`
**Stop all development services**

Stops all running services and cleans up processes.

```bash
make stop
```

**What it does:**
1. Kills cargo run processes (backend)
2. Kills npm run dev processes (frontend)
3. Stops Docker containers
4. Removes Docker networks

**Use when:** You're done developing or need to restart services.

### `make restart`
**Restart all services**

Equivalent to running `make stop` followed by `make dev-bg`.

```bash
make restart
```

**Use when:** Services are acting up or you've made configuration changes.

### `make smart-restart`
**Smart restart - no recompilation (fastest for port conflicts)**

Kills ports and restarts services without any recompilation. Perfect for most port conflicts.

```bash
make smart-restart
```

**What it does:**
1. Kills all cargo run, vite dev, and npm run dev processes
2. Kills any processes using ports 8080 and 5173
3. Stops Docker containers with --remove-orphans
4. Restarts services using existing compiled binaries

**Use when:** 
- Encountering "port already in use" errors (try this first!)
- Need quickest possible restart
- No code changes made since last compilation

**⚡ Speed:** ~5-10 seconds

### `make fast-restart`
**Fast restart - only recompiles app code (recommended)**

Performs incremental compilation, keeping all dependencies and only rebuilding your application code.

```bash
make fast-restart
```

**What it does:**
1. Kills all processes and clears ports
2. Stops Docker containers with --remove-orphans
3. Runs `cargo clean --package rankchoice-api` (keeps dependencies)
4. Rebuilds only the application code
5. Starts fresh development environment

**Use when:** 
- Made code changes and need fresh compilation
- Want faster rebuild than full clean
- Most common restart scenario

**⚡ Speed:** ~30-60 seconds (depending on code changes)

### `make force-restart`
**Force restart with full clean rebuild (slowest)**

Nuclear option that completely cleans and rebuilds everything including all dependencies.

```bash
make force-restart
```

**What it does:**
1. Kills all processes and clears ports
2. Stops Docker containers with --remove-orphans
3. Runs `cargo clean` (removes everything)
4. Rebuilds all dependencies and application code
5. Starts fresh development environment

**Use when:** 
- Dependency conflicts or corruption
- Debugging mysterious compilation issues
- Need absolutely clean state
- Last resort when other restarts don't work

**⚡ Speed:** ~3-5 minutes (rebuilds all dependencies)

### `make kill-ports`
**Kill processes using development ports (8080, 5173)**

Forcefully kills any processes using the development ports.

```bash
make kill-ports
```

**What it does:**
1. Uses `lsof` to find processes using ports 8080 (backend) and 5173 (frontend)
2. Kills them with `kill -9` (forceful termination)
3. Waits 1 second for cleanup

**Use when:** Processes are stuck on development ports.

## Individual Services

### `make docker-up`
**Start database and supporting services**

Starts only the Docker containers (PostgreSQL, LocalStack, MailHog).

```bash
make docker-up
```

**Services started:**
- PostgreSQL database on port 5432
- LocalStack (AWS services emulation) on port 4566
- MailHog (email testing) on ports 1025/8025

**Use when:** You only need the supporting services running.

### `make docker-down`
**Stop database and supporting services**

Stops all Docker containers.

```bash
make docker-down
```

### `make backend`
**Start the backend server (blocking)**

Starts the Rust backend server and shows output in terminal.

```bash
make backend
```

**What it does:**
1. Changes to backend directory
2. Runs `cargo run`
3. Shows compilation and runtime output

**Use when:** You want to see backend logs directly or debug backend issues.

### `make backend-bg`
**Start the backend server in background**

Starts the backend server in background mode.

```bash
make backend-bg
```

**Output:** Logs are written to `logs/backend.log`

### `make frontend`
**Start the frontend development server (blocking)**

Starts the Svelte frontend server and shows output in terminal.

```bash
make frontend
```

**What it does:**
1. Changes to frontend directory
2. Runs `npm run dev`
3. Shows Vite development server output

**Use when:** You want to see frontend build output or debug frontend issues.

### `make frontend-bg`
**Start the frontend development server in background**

Starts the frontend server in background mode.

```bash
make frontend-bg
```

**Output:** Logs are written to `logs/frontend.log`

## Installation & Setup

### `make install`
**Install all dependencies**

Installs dependencies for both backend and frontend.

```bash
make install
```

**What it does:**
1. Installs Rust dependencies (`cargo build` in backend)
2. Installs Node.js dependencies (`npm install` in frontend)
3. Creates logs directory

**Use when:** First time setting up the project or after pulling new dependencies.

### `make install-backend`
**Install backend dependencies**

Installs only Rust dependencies.

```bash
make install-backend
```

### `make install-frontend`
**Install frontend dependencies**

Installs only Node.js dependencies.

```bash
make install-frontend
```

### `make setup-logs`
**Create logs directory**

Creates the `logs/` directory for storing service logs.

```bash
make setup-logs
```

### `make check-deps`
**Check if all required tools are installed**

Verifies that all necessary development tools are available.

```bash
make check-deps
```

**Tools checked:**
- Rust/Cargo
- Node.js/npm
- Docker
- Docker Compose

**Use when:** Setting up on a new machine or troubleshooting environment issues.

## Database Management

### `make db-migrate`
**Run database migrations**

Runs all pending database migrations.

```bash
make db-migrate
```

**Prerequisites:** Database must be running (`make docker-up`)

**What it does:**
1. Changes to backend directory
2. Runs `sqlx migrate run`

**Use when:** 
- First time setup
- After pulling new migrations
- When database schema changes

### `make db-reset`
**Reset database (drop and recreate)**

Completely resets the database by destroying and recreating it.

```bash
make db-reset
```

**What it does:**
1. Stops PostgreSQL container
2. Removes database volume
3. Starts PostgreSQL container
4. Runs migrations

**⚠️ Warning:** This will delete all data in the database.

**Use when:** You need a clean database state for testing.

## Building & Testing

### `make build`
**Build both frontend and backend for production**

Creates optimized production builds.

```bash
make build
```

**What it does:**
1. Builds backend with `cargo build --release`
2. Builds frontend with `npm run build`

**Use when:** Preparing for deployment or testing production builds.

### `make build-backend`
**Build backend for production**

Creates optimized Rust binary.

```bash
make build-backend
```

**Output:** Binary created in `backend/target/release/`

### `make build-frontend`
**Build frontend for production**

Creates optimized static files.

```bash
make build-frontend
```

**Output:** Static files created in `frontend/build/`

### `make test`
**Run all tests**

Runs test suites for both backend and frontend.

```bash
make test
```

### `make test-backend`
**Run backend tests**

Runs Rust tests only.

```bash
make test-backend
```

**What it does:**
1. Changes to backend directory
2. Runs `cargo test`

### `make test-frontend`
**Run frontend tests**

Runs frontend tests only.

```bash
make test-frontend
```

**What it does:**
1. Changes to frontend directory
2. Runs `npm test`

## Utilities & Maintenance

### `make clean`
**Clean all build artifacts and dependencies**

Removes all generated files and dependencies.

```bash
make clean
```

**What it does:**
1. Runs `cargo clean` in backend
2. Removes `node_modules`, `build`, `.svelte-kit` in frontend
3. Removes logs directory
4. Stops and removes Docker volumes

**⚠️ Warning:** This will require reinstalling dependencies.

**Use when:** 
- Freeing up disk space
- Resolving dependency conflicts
- Starting completely fresh

### `make fmt`
**Format code (backend and frontend)**

Formats code according to project standards.

```bash
make fmt
```

**What it does:**
1. Runs `cargo fmt` in backend
2. Runs `npm run format` in frontend

**Use when:** Before committing code or during code reviews.

### `make lint`
**Lint code (backend and frontend)**

Runs linters to check code quality.

```bash
make lint
```

**What it does:**
1. Runs `cargo clippy` in backend
2. Runs `npm run lint` in frontend

**Use when:** Before committing code or during CI/CD.

## Monitoring & Debugging

### `make status`
**Show status of all services**

Displays the current status of all services.

```bash
make status
```

**Output example:**
```
📊 Service Status:
==================
🗄️  Database:  ✅ Running
🦀 Backend:   ✅ Running (http://localhost:8080)
💻 Frontend:  ✅ Running (http://localhost:5173)
```

**Use when:** Checking if services are running properly.

### `make health`
**Check health of all services**

Performs HTTP health checks on running services.

```bash
make health
```

**Output example:**
```
🏥 Health Check:
===============
Backend API: ✅ Healthy
Frontend:    ✅ Healthy
```

**Use when:** Verifying services are responding to requests.

### `make logs`
**Show logs from all services**

Displays recent logs from all services.

```bash
make logs
```

**What it shows:**
1. Last 20 lines from backend logs
2. Last 20 lines from frontend logs
3. Last 10 lines from Docker container logs

**Use when:** Debugging issues or checking service output.

### `make logs-follow`
**Follow logs from all services**

Continuously displays logs as they're generated.

```bash
make logs-follow
```

**Use when:** Monitoring real-time activity across all services.

**Stop with:** `Ctrl+C`

## Production & Deployment

### `make deploy-staging`
**Deploy to staging environment**

Builds and deploys to staging environment.

```bash
make deploy-staging
```

**Prerequisites:** 
- Terraform configured
- AWS credentials set up
- Staging workspace configured

### `make deploy-prod`
**Deploy to production environment**

Builds and deploys to production environment.

```bash
make deploy-prod
```

**Prerequisites:** 
- Terraform configured
- AWS credentials set up
- Production workspace configured

## Common Workflows

### First Time Setup
```bash
# Check if all tools are installed
make check-deps

# Install all dependencies and start services
make quick-start

# Check that everything is working
make health
```

### Daily Development
```bash
# Start development environment
make dev-bg

# Check status
make status

# View logs if needed
make logs

# Stop when done
make stop
```

### After Pulling Changes
```bash
# Stop services
make stop

# Install any new dependencies
make install

# Run new migrations
make db-migrate

# Start services
make dev-bg
```

### Debugging Issues
```bash
# Check service status
make status

# Check health
make health

# View logs
make logs

# Restart if needed
make restart

# For detailed debugging, start services individually
make docker-up
make backend    # See backend output directly
# In another terminal:
make frontend   # See frontend output directly
```

### Testing Before Commit
```bash
# Format code
make fmt

# Lint code
make lint

# Run tests
make test

# Build for production (optional)
make build
```

### Handling Port Conflicts
```bash
# Try these in order (fastest to slowest):

# 1. Smart restart (fastest - no recompilation)
make smart-restart

# 2. Fast restart (if you made code changes)
make fast-restart

# 3. Force restart (if dependencies are problematic)
make force-restart

# OR if you need manual control:
make stop
make kill-ports
make dev-bg

# Check what's using the ports (if issues persist)
lsof -i:8080  # Check backend port
lsof -i:5173  # Check frontend port
```

### Clean Reset
```bash
# Stop everything
make stop

# Clean all artifacts
make clean

# Reinstall and restart
make quick-start
```

## Environment Variables

Some commands respect environment variables:

- `ENVIRONMENT` - Set environment (development, staging, production)
- `DATABASE_URL` - Override database connection string
- `PORT` - Override backend port (default: 8080)

## Port Usage

| Service | Port | URL |
|---------|------|-----|
| Frontend | 5173 | http://localhost:5173 |
| Backend | 8080 | http://localhost:8080 |
| PostgreSQL | 5432 | localhost:5432 |
| MailHog SMTP | 1025 | localhost:1025 |
| MailHog Web | 8025 | http://localhost:8025 |
| LocalStack | 4566 | http://localhost:4566 |

## Troubleshooting

### Common Issues

**Services won't start:**
```bash
make check-deps  # Verify tools are installed
make clean       # Clean everything
make install     # Reinstall dependencies
```

**Database connection errors:**
```bash
make docker-up   # Ensure database is running
make db-reset    # Reset database if corrupted
```

**Port conflicts:**
```bash
# Try these in order (fastest to slowest):
make smart-restart  # Fastest: No recompilation
make fast-restart   # Fast: Only recompile app code  
make force-restart  # Slowest: Full rebuild

# OR manually:
make stop           # Stop all services
make kill-ports     # Kill any lingering processes on ports 8080/5173
make dev-bg         # Restart services
```

**Permission errors:**
```bash
# Ensure Docker is running
# Check file permissions in logs directory
sudo chown -R $USER:$USER logs/
```

### Getting Help

- Run `make help` to see all available commands
- Check service logs with `make logs`
- Verify service status with `make status`
- Test service health with `make health`

## Contributing

When adding new Makefile targets:

1. Add a description after `##`
2. Update this documentation
3. Test on clean environment
4. Consider cross-platform compatibility

Example:
```makefile
new-command: ## Description of what this command does
	@echo "Running new command..."
	# Command implementation
``` 