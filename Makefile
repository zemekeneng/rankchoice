# RankChoice.me Development & Deployment Makefile

.PHONY: help dev dev-bg stop clean install build test docker logs status check-deps \
        force-restart kill-ports smart-restart fast-restart \
        build-lambda deploy bootstrap tf-init tf-plan tf-apply migrate-prod

# Default target
help: ## Show this help message
	@echo "RankChoice.me Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-25s\033[0m %s\n", $$1, $$2}'

# ─── Development Environment ─────────────────────────────────────────────────

dev: check-deps ## Start the full development environment (blocking)
	@echo "🚀 Starting RankChoice.me development environment..."
	@echo "📋 Starting services in order: Database → Backend → Frontend"
	@$(MAKE) docker-up
	@sleep 3
	@echo "🔧 Starting backend and frontend in parallel..."
	@$(MAKE) dev-parallel

dev-bg: check-deps ## Start the full development environment in background
	@echo "🚀 Starting RankChoice.me development environment in background..."
	@$(MAKE) docker-up
	@sleep 3
	@$(MAKE) backend-bg
	@sleep 2
	@$(MAKE) frontend-bg
	@echo "✅ All services started in background"
	@$(MAKE) status

dev-parallel: ## Start backend and frontend in parallel (for internal use)
	@trap 'kill 0' INT; \
	$(MAKE) backend & \
	$(MAKE) frontend & \
	wait

stop: ## Stop all development services
	@echo "🛑 Stopping all development services..."
	@pkill -f "cargo run" || true
	@pkill -f "vite dev" || true
	@pkill -f "npm run dev" || true
	@$(MAKE) kill-ports
	@docker-compose down
	@echo "✅ All services stopped"

kill-ports: ## Kill processes using development ports (8081, 5174)
	@echo "🔫 Killing processes using ports 8081 and 5174..."
	@lsof -ti:8081 | xargs -r kill -9 2>/dev/null || true
	@lsof -ti:5174 | xargs -r kill -9 2>/dev/null || true
	@sleep 1

smart-restart: ## Smart restart - kills ports and restarts without recompilation (fastest)
	@echo "⚡ Smart restarting RankChoice.me (no recompilation)..."
	@echo "🛑 Stopping all services..."
	@pkill -f "cargo run" || true
	@pkill -f "vite dev" || true
	@pkill -f "npm run dev" || true
	@$(MAKE) kill-ports
	@docker-compose down --remove-orphans
	@echo "🚀 Starting services with existing compilation..."
	@$(MAKE) dev-bg
	@echo "✅ Smart restart complete!"

fast-restart: ## Fast restart - only recompiles app code, keeps dependencies
	@echo "🚀 Fast restarting RankChoice.me with incremental compilation..."
	@echo "🛑 Stopping all services..."
	@pkill -f "cargo run" || true
	@pkill -f "vite dev" || true
	@pkill -f "npm run dev" || true
	@$(MAKE) kill-ports
	@docker-compose down --remove-orphans
	@echo "🔧 Incremental rebuild (keeping dependencies)..."
	@cd backend && cargo clean --package rankchoice-api
	@cd backend && cargo build
	@echo "🚀 Starting fresh environment..."
	@$(MAKE) dev-bg
	@echo "✅ Fast restart complete!"

force-restart: ## Force restart with full clean rebuild (slowest)
	@echo "🔄 Force restarting RankChoice.me with full clean compilation..."
	@echo "🛑 Aggressively stopping all services..."
	@pkill -f "cargo run" || true
	@pkill -f "vite dev" || true
	@pkill -f "npm run dev" || true
	@$(MAKE) kill-ports
	@docker-compose down --remove-orphans
	@echo "🧹 Full clean and rebuild (this will take a while)..."
	@cd backend && cargo clean
	@cd backend && cargo build
	@echo "🚀 Starting fresh environment..."
	@$(MAKE) dev-bg
	@echo "✅ Force restart complete!"

# ─── Individual Services ─────────────────────────────────────────────────────

docker-up: ## Start database and supporting services
	@echo "🐳 Starting database and supporting services..."
	@docker-compose up -d postgres localstack mailhog
	@echo "⏳ Waiting for database to be ready..."
	@for i in {1..30}; do \
		if docker exec rankchoice_postgres pg_isready -U rankchoice >/dev/null 2>&1; then \
			echo "✅ Database ready"; \
			exit 0; \
		fi; \
		sleep 1; \
	done; \
	echo "❌ Database failed to start" && exit 1

docker-down: ## Stop database and supporting services
	@echo "🐳 Stopping database and supporting services..."
	@docker-compose down

backend: ## Start the backend server (blocking)
	@echo "🦀 Starting Rust backend server..."
	@cd backend && cargo run

backend-bg: ## Start the backend server in background
	@echo "🦀 Starting Rust backend server in background..."
	@cd backend && cargo run > ../logs/backend.log 2>&1 &
	@sleep 2
	@if pgrep -f "cargo run" > /dev/null; then \
		echo "✅ Backend started successfully"; \
	else \
		echo "❌ Backend failed to start"; \
		exit 1; \
	fi

frontend: ## Start the frontend development server (blocking)
	@echo "💻 Starting Svelte frontend server..."
	@cd frontend && npm run dev

frontend-bg: ## Start the frontend development server in background
	@echo "💻 Starting Svelte frontend server in background..."
	@cd frontend && npm run dev > ../logs/frontend.log 2>&1 &
	@sleep 2
	@if pgrep -f "npm run dev" > /dev/null; then \
		echo "✅ Frontend started successfully"; \
	else \
		echo "❌ Frontend failed to start"; \
		exit 1; \
	fi

# ─── Installation and Setup ─────────────────────────────────────────────────

install: ## Install all dependencies
	@echo "📦 Installing dependencies..."
	@$(MAKE) install-backend
	@$(MAKE) install-frontend
	@$(MAKE) setup-logs
	@echo "✅ All dependencies installed"

install-backend: ## Install backend dependencies
	@echo "🦀 Installing Rust dependencies..."
	@cd backend && cargo build

install-frontend: ## Install frontend dependencies
	@echo "💻 Installing Node.js dependencies..."
	@cd frontend && npm install

setup-logs: ## Create logs directory
	@mkdir -p logs

# ─── Database Management ────────────────────────────────────────────────────

db-migrate: ## Run database migrations (local)
	@echo "🗄️  Running database migrations..."
	@cd backend && sqlx migrate run

db-reset: ## Reset database (drop and recreate)
	@echo "🗄️  Resetting database..."
	@docker-compose down postgres
	@docker volume rm rankchoice_postgres_data || true
	@$(MAKE) docker-up
	@sleep 3
	@$(MAKE) db-migrate

# ─── Building ───────────────────────────────────────────────────────────────

build: ## Build both frontend and backend for production
	@echo "🏗️  Building for production..."
	@$(MAKE) build-backend
	@$(MAKE) build-frontend
	@echo "✅ Build complete"

build-backend: ## Build backend for production (native binary)
	@echo "🦀 Building Rust backend..."
	@cd backend && cargo build --release

build-frontend: ## Build frontend for production
	@echo "💻 Building Svelte frontend..."
	@cd frontend && npm run build

build-lambda: ## Build backend as Lambda deployment package
	@echo "🦀 Building Lambda function..."
	@bash infrastructure/scripts/build-lambda.sh

build-frontend-prod: ## Build frontend with production API URL
	@echo "💻 Building Svelte frontend for production..."
	@cd frontend && VITE_API_URL=https://rankchoice.me/api npm run build

# ─── Infrastructure & Deployment ────────────────────────────────────────────

bootstrap: ## One-time setup: create Terraform state bucket and lock table
	@echo "🏗️  Bootstrapping Terraform state infrastructure..."
	@bash infrastructure/terraform/bootstrap.sh

tf-init: ## Initialize Terraform
	@echo "🔧 Initializing Terraform..."
	@cd infrastructure/terraform && terraform init

tf-plan: ## Preview infrastructure changes
	@echo "📋 Planning Terraform changes..."
	@cd infrastructure/terraform && terraform plan

tf-apply: ## Apply infrastructure changes
	@echo "🚀 Applying Terraform changes..."
	@cd infrastructure/terraform && terraform apply

tf-fmt: ## Format Terraform files
	@echo "🎨 Formatting Terraform files..."
	@cd infrastructure/terraform && terraform fmt -recursive .

tf-output: ## Show Terraform outputs
	@cd infrastructure/terraform && terraform output

deploy: ## Full production deploy: build Lambda + frontend, apply Terraform, sync S3, invalidate CDN
	@echo "🚀 Deploying RankChoice.me to production..."
	@bash infrastructure/scripts/deploy.sh

migrate-prod: ## Run database migrations against Neon (requires DATABASE_URL env var)
	@echo "🗄️  Running migrations against production database..."
	@bash infrastructure/scripts/migrate.sh

# ─── Testing ────────────────────────────────────────────────────────────────

test: ## Run all tests (backend + frontend unit + E2E)
	@echo "🧪 Running all tests..."
	@$(MAKE) test-backend
	@$(MAKE) test-frontend
	@$(MAKE) test-e2e

test-backend: ## Run backend tests
	@echo "🦀 Running Rust tests..."
	@cd backend && cargo test

test-frontend: ## Run frontend tests
	@echo "💻 Running frontend unit tests..."
	@cd frontend && npm run test:unit -- --run

test-e2e: ## Run E2E tests with optimal parallelism (requires dev environment)
	@echo "🧪 Running E2E tests with parallel workers..."
	@echo "⚠️  Ensure dev environment is running (make dev-bg)"
	@cd frontend && npm run test:e2e

test-e2e-fast: ## Run E2E tests with maximum parallelism
	@echo "🚀 Running E2E tests with maximum parallelism..."
	@echo "⚠️  Ensure dev environment is running (make dev-bg)"
	@cd frontend && npx playwright test --workers=4

test-e2e-single: ## Run E2E tests sequentially (safer but slower)
	@echo "🐌 Running E2E tests sequentially..."
	@echo "⚠️  Ensure dev environment is running (make dev-bg)"
	@cd frontend && npx playwright test --workers=1

test-e2e-headed: ## Run E2E tests with browser UI
	@echo "🧪 Running E2E tests (headed mode)..."
	@cd frontend && npm run test:e2e:headed

test-e2e-debug: ## Debug E2E tests interactively
	@echo "🧪 Debugging E2E tests..."
	@cd frontend && npm run test:e2e:debug

test-e2e-static: ## Run E2E tests against static build (production-like)
	@echo "🏗️ Building frontend for static testing..."
	@$(MAKE) build-frontend
	@echo "🧪 Running E2E tests against static server..."
	@cd frontend && npm run test:e2e:static

test-e2e-static-headed: ## Run E2E tests against static build with browser UI
	@echo "🏗️ Building frontend for static testing..."
	@$(MAKE) build-frontend
	@echo "🧪 Running E2E tests against static server (headed mode)..."
	@cd frontend && npm run test:e2e:static:headed

# ─── Utilities ──────────────────────────────────────────────────────────────

clean: ## Clean all build artifacts and dependencies
	@echo "🧹 Cleaning build artifacts..."
	@cd backend && cargo clean
	@cd frontend && rm -rf node_modules build .svelte-kit
	@rm -rf logs
	@docker-compose down -v
	@echo "✅ Clean complete"

logs: ## Show logs from all services
	@echo "📋 Service logs:"
	@echo "=== BACKEND LOGS ==="
	@tail -n 20 logs/backend.log 2>/dev/null || echo "No backend logs found"
	@echo ""
	@echo "=== FRONTEND LOGS ==="
	@tail -n 20 logs/frontend.log 2>/dev/null || echo "No frontend logs found"
	@echo ""
	@echo "=== DOCKER LOGS ==="
	@docker-compose logs --tail=10

logs-follow: ## Follow logs from all services
	@echo "📋 Following logs (Ctrl+C to stop)..."
	@tail -f logs/*.log 2>/dev/null || echo "No logs found. Start services with 'make dev-bg'"

status: ## Show status of all services
	@echo "📊 Service Status:"
	@echo "=================="
	@printf "🗄️  Database:  "
	@if docker ps | grep rankchoice_postgres | grep -q "Up"; then \
		echo "✅ Running"; \
	else \
		echo "❌ Stopped"; \
	fi
	@printf "🦀 Backend:   "
	@if pgrep -f "cargo run" > /dev/null; then \
		echo "✅ Running (http://localhost:8081)"; \
	else \
		echo "❌ Stopped"; \
	fi
	@printf "💻 Frontend:  "
	@if pgrep -f "npm run dev" > /dev/null; then \
		echo "✅ Running (http://localhost:5174)"; \
	else \
		echo "❌ Stopped"; \
	fi
	@echo ""

health: ## Check health of all services
	@echo "🏥 Health Check:"
	@echo "==============="
	@printf "Backend API: "
	@curl -s http://localhost:8081/health > /dev/null && echo "✅ Healthy" || echo "❌ Unhealthy"
	@printf "Frontend:    "
	@curl -s http://localhost:5174 > /dev/null && echo "✅ Healthy" || echo "❌ Unhealthy"

check-deps: ## Check if all required tools are installed
	@echo "🔍 Checking dependencies..."
	@command -v cargo > /dev/null || (echo "❌ Rust/Cargo not found. Install from https://rustup.rs/" && exit 1)
	@command -v node > /dev/null || (echo "❌ Node.js not found. Install from https://nodejs.org/" && exit 1)
	@command -v npm > /dev/null || (echo "❌ npm not found. Install Node.js from https://nodejs.org/" && exit 1)
	@command -v docker > /dev/null || (echo "❌ Docker not found. Install from https://docker.com/" && exit 1)
	@command -v docker-compose > /dev/null || (echo "❌ Docker Compose not found. Install Docker Desktop" && exit 1)
	@echo "✅ All dependencies found"

check-deploy-deps: ## Check if deployment tools are installed
	@echo "🔍 Checking deployment dependencies..."
	@command -v cargo > /dev/null || (echo "❌ Rust/Cargo not found" && exit 1)
	@command -v cargo-lambda > /dev/null || (echo "❌ cargo-lambda not found. Install: brew install cargo-lambda" && exit 1)
	@command -v terraform > /dev/null || (echo "❌ Terraform not found. Install: brew install terraform" && exit 1)
	@command -v aws > /dev/null || (echo "❌ AWS CLI not found. Install: brew install awscli" && exit 1)
	@echo "✅ All deployment dependencies found"

fmt: ## Format code (backend, frontend, and Terraform)
	@echo "🎨 Formatting code..."
	@cd backend && cargo fmt
	@cd frontend && npm run format
	@cd infrastructure/terraform && terraform fmt -recursive .

lint: ## Lint code (backend and frontend)
	@echo "🔍 Linting code..."
	@cd backend && cargo clippy
	@cd frontend && npm run lint

# ─── Quick Workflows ────────────────────────────────────────────────────────

quick-start: install dev-bg ## Quick start: install dependencies and start all services
	@echo ""
	@echo "🎉 RankChoice.me is ready!"
	@echo "🌐 Frontend: http://localhost:5174"
	@echo "🔧 Backend:  http://localhost:8081"
	@echo "📧 MailHog:  http://localhost:8026"
	@echo ""
	@echo "Use 'make stop' to stop all services"
	@echo "Use 'make status' to check service status"
	@echo "Use 'make logs' to view logs"

restart: stop dev-bg ## Restart all services
