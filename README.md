# RankedChoice.me

A modern, secure web application for creating and managing ranked-choice voting polls with real-time results and comprehensive analytics.

![RankedChoice.me](https://img.shields.io/badge/Status-In%20Development-yellow)
![License](https://img.shields.io/badge/License-MIT-blue)
![Backend](https://img.shields.io/badge/Backend-Rust%20%2B%20Axum-orange)
![Frontend](https://img.shields.io/badge/Frontend-SvelteKit%20%2B%20Svelte%205-red)
![Database](https://img.shields.io/badge/Database-PostgreSQL-blue)

## 🗳️ What is Ranked-Choice Voting?

Ranked-choice voting (RCV) is an electoral system where voters rank candidates in order of preference. Winners are determined through instant runoff rounds:
- **Single-winner**: Candidate needs >50% of votes to win
- **Multi-winner**: Multiple candidates can win based on vote thresholds

## ✨ Features

### Core Functionality
- 🗳️ **Create RCV Polls** - Single-winner and multi-winner elections
- 👥 **Candidate Management** - Add, edit, reorder candidates with descriptions
- 🔗 **Secure Voting Links** - Unique ballot tokens for each voter
- 📊 **Real-time Results** - Live vote tabulation with round-by-round visualization
- 📱 **Mobile-Responsive** - Optimized for all devices

### Authentication & Security
- 🔐 **JWT Authentication** - Secure user registration and login
- 🛡️ **Route Protection** - Authenticated-only access to sensitive areas
- 🔑 **Password Security** - Argon2 hashing for maximum security
- 🚫 **Fraud Prevention** - Ballot validation and duplicate vote protection

### User Experience
- 🎨 **Modern UI** - Beautiful Tailwind CSS design with Svelte 5
- ⚡ **Fast Performance** - Optimized Rust backend and reactive frontend
- 📋 **Form Validation** - Real-time validation with helpful error messages
- 🔄 **Poll Preview** - Preview polls before publishing
- 📈 **Dashboard** - Comprehensive poll management interface

### Developer Experience
- 🧪 **Comprehensive Testing** - 59 backend tests + 17 E2E tests
- 🛠️ **Development Tools** - Automated Makefile workflows
- 📚 **Complete Documentation** - API specs, guides, and examples
- 🐳 **Docker Setup** - Containerized development environment

## 🏗️ Architecture

### Tech Stack
- **Backend**: Rust with Axum framework
- **Frontend**: SvelteKit with Svelte 5 and TypeScript
- **Database**: PostgreSQL with SQLx migrations
- **Styling**: Tailwind CSS
- **Testing**: Rust tests + Vitest + Playwright E2E
- **Development**: Docker Compose for local services

### Project Structure
```
rankedchoice/
├── backend/           # Rust API server (Axum)
│   ├── src/          # Source code
│   ├── migrations/   # Database migrations
│   └── tests/        # Backend tests
├── frontend/         # SvelteKit app (Svelte 5 + TypeScript)
│   ├── src/          # Source code
│   └── e2e/          # E2E tests
├── docs/             # Documentation
├── infrastructure/   # Terraform for AWS deployment
└── Makefile         # Development automation
```

## 🚀 Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (18+)
- [Docker](https://docker.com/) & Docker Compose
- [Make](https://www.gnu.org/software/make/) (usually pre-installed)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/rankedchoice.git
   cd rankedchoice
   ```

2. **Start the development environment**
   ```bash
   make dev-bg
   ```
   
   *For first-time setup with dependency installation:*
   ```bash
   make quick-start
   ```

3. **Verify everything is running**
   ```bash
   make status
   make health
   ```

4. **Access the application**
   - Frontend: http://localhost:5173
   - Backend API: http://localhost:8080
   - MailHog (email testing): http://localhost:8025

> 📖 **Important**: Follow the [Development Process](docs/DEVELOPMENT_PROCESS.md) for the complete standard workflow. This prevents common issues like 404 errors and service conflicts.

### Development Commands

```bash
# Start development environment (background)
make dev-bg

# Check service status
make status

# View logs
make logs

# Run all tests
make test

# Run E2E tests
make test-e2e

# Stop all services
make stop
```

For detailed commands, see [`docs/MAKEFILE_COMMANDS.md`](docs/MAKEFILE_COMMANDS.md).

## 📖 Documentation

This project maintains comprehensive documentation organized by purpose and audience:

### 🚀 Getting Started (Developers)
| Document | Purpose | Audience |
|----------|---------|----------|
| **[Quick Start](#-quick-start)** | Essential setup and development commands | All developers (START HERE) |
| **[Development Process](docs/DEVELOPMENT_PROCESS.md)** | Complete standard workflow | All developers |
| **[Makefile Commands](docs/MAKEFILE_COMMANDS.md)** | Full command reference | All developers |

### 📋 Project Information (Product & Planning)
| Document | Purpose | Audience |
|----------|---------|----------|
| **[Development Status](DEVELOPMENT_STATUS.md)** | Current implementation status and next steps | Product managers, developers |
| **[Project Planning](PLANNING.md)** | Complete project specification and architecture | Product managers, architects |

### 🔧 Technical Documentation (Implementation)
| Document | Purpose | Audience |
|----------|---------|----------|
| **[API Specification](docs/API_SPECIFICATION.md)** | Complete REST API documentation | Backend developers, frontend developers |
| **[RCV Algorithm](docs/RCV_ALGORITHM.md)** | Ranked-choice voting implementation details | Algorithm developers |
| **[Svelte 5 Guide](docs/SVELTE5_GUIDE.md)** | Frontend development patterns and syntax | Frontend developers |

### 🧪 Testing Documentation (Quality Assurance)
| Document | Purpose | Audience |
|----------|---------|----------|
| **[Backend Testing Guide](backend/docs/TESTING_GUIDE.md)** | Backend testing strategy and implementation | Backend developers, QA |
| **[E2E Testing Guide](frontend/e2e/README.md)** | End-to-end testing comprehensive guide | All developers, QA |

### 🔌 Service Documentation (Microservices)
| Document | Purpose | Audience |
|----------|---------|----------|
| **[Email Service](services/email/README.md)** | Email microservice setup and API | DevOps, backend developers |

## 🧪 Testing

### Backend Tests (59 tests)
- **Authentication**: Registration, login, JWT validation (18 tests)
- **Polls API**: CRUD operations, validation (14 tests) 
- **Candidates API**: Management and ordering (11 tests)
- **RCV Algorithm**: Vote tabulation and tie-breaking (5 tests)
- **Voting System**: Ballot submission and validation (4 tests)
- **Results API**: Real-time results and rounds (5 tests)
- **Integration**: End-to-end workflows (2 tests)

### Frontend Tests
- **Unit Tests**: Component and utility testing with Vitest
- **E2E Tests**: 17 comprehensive Playwright tests covering:
  - Authentication flows (6 tests)
  - Poll management (8 tests)
  - Integration journeys (2 tests)
  - Complete user workflows from registration to poll creation

### Running Tests

```bash
# All tests (backend + frontend + E2E)
make test

# Backend only
make test-backend

# Frontend unit tests only
make test-frontend

# E2E tests only
make test-e2e

# E2E with visual debugging
make test-e2e-headed
```

## 🔧 Development

### Environment Setup
1. Ensure prerequisites are installed
2. Run `make check-deps` to verify tools
3. Run `make quick-start` for full setup
4. Use `make dev-bg` for daily development

### Database Management
```bash
# Run migrations
make db-migrate

# Reset database (clean slate)
make db-reset
```

### Common Workflows
```bash
# After pulling changes
make stop && make install && make db-migrate && make dev-bg

# Debugging issues
make status && make health && make logs

# Before committing
make fmt && make lint && make test
```

### Troubleshooting
| Problem | Solution |
|---------|----------|
| 404 errors | `make smart-restart` |
| Port conflicts | `make kill-ports && make dev-bg` |
| Code not updating | `make fast-restart` |
| Dependency issues | `make force-restart` |

> 🛡️ **The Golden Rule**: ALWAYS use `make` commands. Never start services manually.

## 🚧 Current Status

### ✅ Completed Features
- Complete backend API with authentication
- RCV tabulation engine with tie-breaking
- User registration and login system
- Poll creation and management interface
- Dashboard with poll listing
- Comprehensive test coverage (76 total tests)
- Development environment automation

### 🔄 In Progress
- Voting interface with drag-and-drop ranking
- Email distribution system
- Results visualization enhancements

### 📋 Planned
- Real-time updates with WebSockets
- Advanced analytics dashboard
- Social media integration
- AWS deployment with Terraform

See [`DEVELOPMENT_STATUS.md`](DEVELOPMENT_STATUS.md) for detailed progress.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`make test`)
5. Format code (`make fmt`)
6. Commit changes (`git commit -m 'Add amazing feature'`)
7. Push to branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Development Guidelines
- Follow existing code patterns and naming conventions
- Add tests for new functionality
- Update documentation for API changes
- Use the provided Makefile commands for consistency

## 📊 Performance

- **API Response Time**: < 200ms (p95)
- **Frontend Load Time**: < 2 seconds
- **Database Queries**: Optimized with proper indexing
- **Test Coverage**: 59 backend tests + 17 E2E tests
- **Build Time**: ~30-60 seconds incremental

## 🔒 Security

- **Authentication**: JWT tokens with Argon2 password hashing
- **Authorization**: Role-based access control (RBAC)
- **Input Validation**: Comprehensive request validation
- **SQL Injection**: Protection via SQLx prepared statements
- **Rate Limiting**: Planned for production deployment
- **HTTPS**: Required for production deployment

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://rust-lang.org/) and [Axum](https://github.com/tokio-rs/axum)
- Frontend powered by [SvelteKit](https://kit.svelte.dev/) and [Svelte 5](https://svelte-5-preview.vercel.app/)
- Styled with [Tailwind CSS](https://tailwindcss.com/)
- Tested with [Playwright](https://playwright.dev/)
- Deployed on [AWS](https://aws.amazon.com/) (planned)

## 📞 Support

- 📧 **Email**: [Insert contact email]
- 🐛 **Issues**: [GitHub Issues](https://github.com/your-org/rankedchoice/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/your-org/rankedchoice/discussions)

---

**Made with ❤️ for better democracy through ranked-choice voting** 