# RankedChoice.me - Standard Development Process

This document defines the **official standard development process** for RankedChoice.me. Following these steps ensures consistent, reliable development for both human and AI developers.

## 🎯 The Golden Rule

**ALWAYS use the Makefile commands for development tasks.** Never start services manually.

## 🚀 Standard Development Workflow

### 1. Fresh Development Session Start

```bash
# Single command to start everything
make dev-bg
```

**What this does:**
- ✅ Checks all dependencies (Rust, Node.js, Docker)
- ✅ Starts PostgreSQL database in Docker
- ✅ Waits for database to be ready (up to 30 seconds)
- ✅ Starts Rust backend on port 8080
- ✅ Starts SvelteKit frontend on port 5173
- ✅ Displays service status

**Expected output:**
```
🚀 Starting RankedChoice.me development environment in background...
🐳 Starting database and supporting services...
⏳ Waiting for database to be ready...
✅ Database ready
🦀 Starting Rust backend server in background...
✅ Backend started successfully
💻 Starting Svelte frontend server in background...
✅ Frontend started successfully
✅ All services started in background
```

### 2. Verify Everything is Running

```bash
make status
```

**Expected output:**
```
📊 Service Status:
==================
🗄️  Database:  ✅ Running
🦀 Backend:   ✅ Running (http://localhost:8080)
💻 Frontend:  ✅ Running (http://localhost:5173)
```

### 3. Health Check

```bash
make health
```

**Expected output:**
```
🏥 Health Check:
===============
Backend API: ✅ Healthy
Frontend:    ✅ Healthy
```

### 4. Access the Application

- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:8080
- **Email Testing (MailHog)**: http://localhost:8025

### 5. End Development Session

```bash
make stop
```

## 🔧 Troubleshooting & Restart Options

### When to Use Each Restart Command

| Situation | Command | Use Case |
|-----------|---------|----------|
| **Port conflicts or service hangs** | `make smart-restart` | Fastest - no recompilation |
| **Code changes not reflecting** | `make fast-restart` | Recommended - recompiles app only |
| **Dependency issues or corruption** | `make force-restart` | Slowest - full clean rebuild |

### Common Issues & Solutions

#### ❌ "404 Not Found" Errors
**Problem**: Backend routes returning 404
**Solution**: Backend likely not running or outdated

```bash
make smart-restart
```

#### ❌ Database Connection Errors
**Problem**: Backend hanging on startup
**Solution**: Database not ready

```bash
make stop
make dev-bg
```

#### ❌ Port Already in Use
**Problem**: Services won't start due to port conflicts
**Solution**: Kill processes using development ports

```bash
make kill-ports
make dev-bg
```

#### ❌ Compilation Errors
**Problem**: Code changes breaking builds
**Solution**: Clean rebuild

```bash
make force-restart
```

## 📋 Development Commands Reference

### Essential Commands
```bash
make dev-bg          # Start all services (primary command)
make stop            # Stop all services
make status          # Check service status
make health          # Check service health
make logs            # View recent logs
make restart         # Full restart (stop + dev-bg)
```

### Advanced Commands
```bash
make smart-restart   # Fastest restart (no recompilation)
make fast-restart    # Recompile app code only
make force-restart   # Full clean rebuild
make kill-ports      # Kill processes on ports 8080, 5173
make logs-follow     # Follow logs in real-time
```

### Testing Commands
```bash
make test            # Run all tests
make test-backend    # Run Rust tests only
make test-frontend   # Run frontend unit tests
make test-e2e        # Run E2E tests (requires dev-bg)
```

## 🎯 Standards for AI Developers

When working on RankedChoice.me:

1. **Always start with**: `make dev-bg`
2. **Always verify with**: `make status` and `make health`
3. **If errors occur**: Use appropriate restart command from table above
4. **Before making changes**: Ensure all services are healthy
5. **When debugging**: Use `make logs` to check service output
6. **When finished**: Run `make stop` to clean up

## ⚠️ Common Anti-Patterns to Avoid

### ❌ DON'T DO THIS:
```bash
# Manual service startup (WRONG)
cd backend && cargo run &
cd frontend && npm run dev &
docker-compose up postgres &
```

### ✅ DO THIS INSTEAD:
```bash
# Standardized startup (CORRECT)
make dev-bg
```

### ❌ DON'T DO THIS:
```bash
# Manual debugging of port conflicts (WRONG)
lsof -ti:8080 | xargs kill -9
lsof -ti:5173 | xargs kill -9
```

### ✅ DO THIS INSTEAD:
```bash
# Standardized port cleanup (CORRECT)
make kill-ports
make dev-bg
```

## 🔄 Development Cycle

1. **Start Session**: `make dev-bg`
2. **Verify Health**: `make status` && `make health`
3. **Develop**: Make code changes
4. **Test**: `make test-backend` or `make test-e2e`
5. **Restart if needed**: `make smart-restart` or `make fast-restart`
6. **End Session**: `make stop`

## 📊 Service Dependencies

```
Frontend (5173) ──depends on──> Backend (8080)
                                     │
Backend (8080)  ──depends on──> PostgreSQL (5432)
                                     │
All Services    ──orchestrated by──> Makefile
```

## 🎉 Success Indicators

When everything is working correctly:

1. **`make dev-bg`** completes without errors
2. **`make status`** shows all services ✅ Running
3. **`make health`** shows all services ✅ Healthy
4. **Frontend loads** at http://localhost:5173
5. **Backend responds** to http://localhost:8080/health
6. **Features work** (e.g., anonymous voting, poll creation)

---

**Remember**: The Makefile is the single source of truth for development workflows. When in doubt, use `make help` to see all available commands.