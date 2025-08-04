# 🚀 RankChoice.app - Developer Quick Start

## One-Command Development Setup

```bash
make dev-bg
```

## Verify Success

```bash
make status    # Should show all services ✅ Running
make health    # Should show all services ✅ Healthy
```

## Access Points

- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:8080  
- **Email Testing**: http://localhost:8025

## Stop Development

```bash
make stop
```

## Common Issues & Solutions

| Problem | Solution |
|---------|----------|
| 404 errors | `make smart-restart` |
| Port conflicts | `make kill-ports && make dev-bg` |
| Code not updating | `make fast-restart` |
| Dependency issues | `make force-restart` |

## First Time Setup

```bash
git clone <repo>
cd rankchoice
make quick-start  # Installs deps + starts services
```

## The Golden Rule

**ALWAYS use `make` commands. Never start services manually.**

📖 **Full documentation**: [Development Process](docs/DEVELOPMENT_PROCESS.md)