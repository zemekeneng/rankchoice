#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TF_DIR="${PROJECT_ROOT}/infrastructure/terraform"

# Load .env if present
if [ -f "${PROJECT_ROOT}/.env" ]; then
    set -a
    source "${PROJECT_ROOT}/.env"
    set +a
fi

# Wire NEON_DATABASE_URL into Terraform
if [ -n "${NEON_DATABASE_URL:-}" ]; then
    export TF_VAR_neon_database_url="${NEON_DATABASE_URL}"
elif [ -z "${TF_VAR_neon_database_url:-}" ]; then
    echo "Error: NEON_DATABASE_URL (in .env) or TF_VAR_neon_database_url must be set"
    exit 1
fi

echo "=== Step 1: Build Lambda ==="
bash "${SCRIPT_DIR}/build-lambda.sh"

echo ""
echo "=== Step 2: Build Frontend ==="
cd "${PROJECT_ROOT}/frontend"
VITE_API_URL="https://rankchoice.me/api" npm run build

echo ""
echo "=== Step 3: Terraform Apply ==="
cd "${TF_DIR}"
terraform init
terraform apply

echo ""
echo "=== Step 4: Sync Frontend to S3 ==="
BUCKET=$(terraform output -raw static_bucket)
aws s3 sync "${PROJECT_ROOT}/frontend/build/" "s3://${BUCKET}/" --delete

echo ""
echo "=== Step 5: Invalidate CloudFront Cache ==="
DISTRIBUTION_ID=$(terraform output -raw cloudfront_distribution_id)
aws cloudfront create-invalidation \
  --distribution-id "${DISTRIBUTION_ID}" \
  --paths "/*"

echo ""
echo "=== Deploy Complete ==="
echo "Site: https://rankchoice.me"
