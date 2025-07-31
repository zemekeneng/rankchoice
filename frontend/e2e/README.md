# End-to-End Tests

This directory contains comprehensive Playwright E2E tests that verify the complete user flows of the RankChoice application.

## Test Files

### `auth.test.ts`
Tests authentication functionality:
- User registration and login
- Form validation
- Route protection
- Session persistence
- Logout functionality

### `polls.test.ts`
Tests poll creation and management:
- Poll creation (single-winner and multi-winner)
- Form validation
- Candidate management (add, remove, reorder)
- Poll preview functionality
- Dashboard display
- Date formatting
- Data persistence

### `integration.test.ts`
Tests complete user journeys:
- Full registration → login → poll creation → dashboard flow
- Multiple poll creation
- Session persistence across login/logout
- Error handling and recovery

## Running Tests

### Prerequisites
Ensure both backend and frontend are running:
```bash
# From project root
make dev-bg
```

### Test Commands

```bash
# Run all E2E tests (headless)
npm run test:e2e

# Run tests with browser UI visible
npm run test:e2e:headed

# Debug tests interactively
npm run test:e2e:debug

# View test report after running
npm run test:e2e:report

# Run specific test file
npx playwright test auth.test.ts

# Run specific test
npx playwright test --grep "should register a new user"
```

### Via Makefile
```bash
# Run E2E tests (from project root)
make test-e2e

# Run all tests including backend
make test
```

## Test Strategy

### Authentication Tests
- ✅ Registration with valid data
- ✅ Login with existing credentials  
- ✅ Form validation errors
- ✅ Route protection (redirects)
- ✅ Logout functionality
- ✅ Invalid credentials handling

### Poll Management Tests
- ✅ Empty state for new users
- ✅ Single-winner poll creation
- ✅ Multi-winner poll creation
- ✅ Form validation (title, candidates)
- ✅ Candidate management (CRUD operations)
- ✅ Poll preview functionality
- ✅ Dashboard display and navigation
- ✅ Date formatting verification
- ✅ Data persistence after refresh

### Integration Tests
- ✅ Complete user journey (20+ steps)
- ✅ Multiple poll creation
- ✅ Cross-session persistence
- ✅ Error recovery workflows
- ✅ Authentication state management

## Test Data

Tests use unique identifiers (timestamps) to avoid conflicts:
- Email: `test-${timestamp}@example.com`
- Poll titles: `Test Poll ${timestamp}`

This ensures tests can run in parallel and don't interfere with each other.

## Configuration

Tests are configured in `playwright.config.ts`:
- Base URL: `http://localhost:5173`
- Timeout: 10s actions, 30s navigation
- Retries: 2 attempts for flaky tests
- Browser: Chromium (Desktop Chrome)
- Traces: Captured on first retry for debugging

## Debugging Tests

### Visual Debugging
```bash
npm run test:e2e:headed
```

### Interactive Debugging
```bash
npm run test:e2e:debug
```

### View Test Reports
```bash
npm run test:e2e:report
```

### Screenshots and Videos
Failed tests automatically capture:
- Screenshots at failure point
- Full page traces
- Video recordings (in CI)

## Continuous Integration

Tests are designed to run in CI environments:
- Database reset before test runs
- Unique test data to avoid conflicts
- Retry logic for network flakiness
- Comprehensive error reporting

## Coverage

The E2E tests verify:
- ✅ User authentication flows
- ✅ Poll creation and management
- ✅ Form validation and error handling
- ✅ Data persistence and display
- ✅ Navigation and routing
- ✅ Success/error messaging
- ✅ Responsive UI behavior

These tests complement the unit tests and provide confidence that the complete application works as expected from a user's perspective. 