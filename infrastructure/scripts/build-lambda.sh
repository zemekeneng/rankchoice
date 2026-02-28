#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${PROJECT_ROOT}/backend"

if ! command -v cargo-lambda &> /dev/null; then
    echo "Error: cargo-lambda is not installed."
    echo "Install with: brew install cargo-lambda"
    echo "         or: pip3 install cargo-lambda"
    exit 1
fi

echo "Building Lambda function..."
cargo lambda build --release --features lambda

LAMBDA_DIR="${PROJECT_ROOT}/backend/target/lambda/rankchoice-api"
BOOTSTRAP="${LAMBDA_DIR}/bootstrap"
if [ ! -f "${BOOTSTRAP}" ]; then
    echo "Error: Bootstrap binary not found at ${BOOTSTRAP}"
    exit 1
fi

ZIP_PATH="${LAMBDA_DIR}/bootstrap.zip"
echo "Packaging bootstrap binary..."
cd "${LAMBDA_DIR}"
zip -j "${ZIP_PATH}" bootstrap

echo "Lambda package created: ${ZIP_PATH}"
