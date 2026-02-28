#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Load .env if present
if [ -f "${PROJECT_ROOT}/.env" ]; then
    set -a
    source "${PROJECT_ROOT}/.env"
    set +a
fi

# Use NEON_DATABASE_URL for production migrations
if [ -n "${NEON_DATABASE_URL:-}" ]; then
    export DATABASE_URL="${NEON_DATABASE_URL}"
fi

if [ -z "${DATABASE_URL:-}" ]; then
    echo "Error: NEON_DATABASE_URL (in .env) or DATABASE_URL must be set"
    exit 1
fi

cd "${PROJECT_ROOT}/backend"

echo "Running SQLx migrations..."
sqlx migrate run --source ./migrations

echo "Migrations complete!"
