#!/bin/bash

# RankChoice.app Project Setup Script

echo "🚀 Setting up RankChoice.app project structure..."

# Create backend structure
echo "📦 Creating backend (Rust) structure..."
mkdir -p backend/{src/{api,models,services,rcv,middleware,utils},migrations,tests}

# Initialize Rust project
cd backend
cargo init --name rankchoice-api
cd ..

# Create frontend structure
echo "📦 Creating frontend (SvelteKit) structure..."
npm create svelte@latest frontend -- --template=skeleton --types=typescript --no-add-css --no-add-eslint --no-add-prettier --no-add-playwright --no-add-vitest

# Create services structure
echo "📦 Creating services structure..."
mkdir -p services/email/{src,dist}

# Create infrastructure structure
echo "📦 Creating infrastructure structure..."
mkdir -p infrastructure/terraform/{modules,environments/{dev,staging,prod}}

# Create Docker setup
echo "🐳 Creating Docker configuration..."
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    container_name: rankchoice_postgres
    environment:
      POSTGRES_DB: rankchoice
      POSTGRES_USER: rankchoice
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rankchoice"]
      interval: 10s
      timeout: 5s
      retries: 5

  localstack:
    image: localstack/localstack:latest
    container_name: rankchoice_localstack
    ports:
      - "4566:4566"
    environment:
      - SERVICES=s3,sqs,ses,dynamodb,secretsmanager
      - DEBUG=1
      - DATA_DIR=/tmp/localstack/data
    volumes:
      - localstack_data:/tmp/localstack
      - /var/run/docker.sock:/var/run/docker.sock

  mailhog:
    image: mailhog/mailhog:latest
    container_name: rankchoice_mailhog
    ports:
      - "1025:1025"
      - "8025:8025"

volumes:
  postgres_data:
  localstack_data:
EOF

# Create backend Cargo.toml
echo "⚙️ Configuring backend dependencies..."
cat > backend/Cargo.toml << 'EOF'
[package]
name = "rankchoice-api"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
hyper = "1.0"
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }

# Authentication
jsonwebtoken = "9.2"
argon2 = "0.5"
rand = "0.8"

# AWS Lambda
lambda_runtime = { version = "0.8", optional = true }
lambda_web = { version = "0.2", optional = true }
aws-config = "1.0"
aws-sdk-s3 = "1.0"
aws-sdk-ses = "1.0"
aws-sdk-sqs = "1.0"
aws-sdk-dynamodb = "1.0"

# Utilities
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1.0"
anyhow = "1.0"

[features]
default = []
lambda = ["lambda_runtime", "lambda_web"]

[dev-dependencies]
mockall = "0.12"
proptest = "1.4"

[[bin]]
name = "rankchoice-api"
path = "src/main.rs"
EOF

# Create frontend package.json additions
echo "⚙️ Configuring frontend dependencies..."
cd frontend
npm install -D @tailwindcss/forms @tailwindcss/typography tailwindcss autoprefixer postcss
npm install -D @types/node @sveltejs/adapter-static
# Ensure Svelte 5 is installed
npm install svelte@next @sveltejs/kit@next @sveltejs/vite-plugin-svelte@next
npm install svelte-dnd-action axios zod @tanstack/svelte-query

# Create tailwind config
cat > tailwind.config.js << 'EOF'
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
  ],
}
EOF

# Create postcss config
cat > postcss.config.js << 'EOF'
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
EOF

# Create app.css
mkdir -p src
cat > src/app.css << 'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;
EOF

cd ..

# Create basic backend structure files
echo "📝 Creating initial backend files..."
cat > backend/src/main.rs << 'EOF'
use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .layer(CorsLayer::permissive());

    // Run our app with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
EOF

# Create initial migration
mkdir -p backend/migrations
cat > backend/migrations/001_initial_schema.sql << 'EOF'
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    role VARCHAR(50) NOT NULL DEFAULT 'pollster',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
EOF

# Create GitHub Actions workflow
echo "🔧 Creating CI/CD pipeline..."
mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  backend-test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    - uses: Swatinem/rust-cache@v2
      with:
        workspaces: backend
    - name: Run tests
      working-directory: backend
      run: cargo test --verbose

  frontend-test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions/setup-node@v3
      with:
        node-version: 18
        cache: npm
        cache-dependency-path: frontend/package-lock.json
    - name: Install dependencies
      working-directory: frontend
      run: npm ci
    - name: Type check
      working-directory: frontend
      run: npm run check
EOF

# Create .gitignore
cat > .gitignore << 'EOF'
# Dependencies
node_modules/
target/
dist/

# Environment
.env
.env.local
.env.*.local

# IDE
.vscode/
.idea/
*.swp
*.swo
.DS_Store

# Build outputs
*.log
*.pid
*.seed
*.pid.lock

# Test coverage
coverage/
*.lcov
.nyc_output/

# Terraform
*.tfstate
*.tfstate.*
.terraform/
.terraform.lock.hcl

# AWS
.aws/

# Database
*.sql.bak
postgres_data/

# Misc
tmp/
temp/
EOF

echo "✅ Project structure created successfully!"
echo ""
echo "Next steps:"
echo "1. Make the script executable: chmod +x setup.sh"
echo "2. Install frontend dependencies: cd frontend && npm install"
echo "3. Start the database: docker-compose up -d postgres"
echo "4. Run backend migrations: cd backend && sqlx migrate run"
echo "5. Start the backend: cd backend && cargo run"
echo "6. Start the frontend: cd frontend && npm run dev"
echo ""
echo "🎉 Happy coding!" 