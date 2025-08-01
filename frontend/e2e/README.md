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

### `voters.test.ts`
Tests voter management and statistics:
- Adding voters to polls
- Voter status tracking (invited/voted/pending)
- Voter statistics display
- Ballot token generation
- Comprehensive voter management workflows

### `voter-stats-fix.test.ts` & `voter-stats-simple.test.ts`
Tests for specific voter statistics bug fixes:
- Proper display of voter counts and statistics
- Backend/frontend data format consistency
- Bug regression prevention

### `homepage.test.ts` ⭐ **NEW**
Tests homepage and landing page functionality:
- Hero section with authentication-aware content
- Features section display and navigation
- Call-to-action buttons and flows
- How it works section
- Responsive design on mobile
- Navigation between authenticated/unauthenticated states

### `voting-interface.test.ts` ⭐ **NEW**
Tests comprehensive voting interface and drag-and-drop ranking:
- Voting interface display and poll information
- Drag-and-drop candidate ranking functionality
- Rank/unrank candidate buttons
- Ballot submission and validation
- Voting receipt display and verification
- Double-voting prevention
- Error handling for invalid tokens
- Mobile viewport compatibility
- Loading states and empty states

### `complete-workflow.test.ts` ⭐ **NEW**
Tests complete end-to-end poll workflow:
- Full lifecycle: Registration → Poll Creation → Voter Management → Voting → Results
- Multi-voter simulation with different preferences
- Results verification after voting
- Creator and voter perspective testing
- Cross-session persistence and authentication
- Error handling throughout the workflow

### `enhanced-forms.test.ts` ⭐ **NEW**
Tests enhanced form features and interactions:
- Form actions (cancel, preview, submit)
- Error handling and validation states
- Loading states and accessibility
- Poll type selection (single/multi-winner)
- Candidate management with dynamic inputs
- Form state management and navigation

## Test ID Coverage

All tests now use comprehensive `data-testid` attributes for reliable element targeting:

### **Navigation & Layout**
- `data-testid="home-link"` - Main logo/home link
- `data-testid="login-link"` - Login navigation link
- `data-testid="register-link"` - Register navigation link
- `data-testid="dashboard-btn"` - Dashboard navigation button
- `data-testid="logout-btn"` - Logout button
- `data-testid="welcome-text"` - Welcome message with user name

### **Authentication Forms**
- `data-testid="login-heading"` - Login page heading
- `data-testid="email-input"` - Login email input
- `data-testid="password-input"` - Login password input
- `data-testid="login-submit-btn"` - Login submit button
- `data-testid="login-error"` - Login error message
- `data-testid="register-heading"` - Registration page heading
- `data-testid="name-input"` - Registration name input
- `data-testid="register-email-input"` - Registration email input
- `data-testid="register-password-input"` - Registration password input
- `data-testid="confirm-password-input"` - Confirm password input
- `data-testid="register-submit-btn"` - Registration submit button
- `data-testid="register-error"` - Registration error message

### **Homepage/Landing Page**
- `data-testid="hero-section"` - Hero section container
- `data-testid="hero-title"` - Main title
- `data-testid="hero-get-started-btn"` - Get started button
- `data-testid="hero-sign-in-btn"` - Sign in button
- `data-testid="features-section"` - Features section
- `data-testid="feature-*"` - Individual feature cards
- `data-testid="cta-section"` - Call-to-action section

### **Dashboard**
- `data-testid="create-poll-btn"` - Main create poll button
- `data-testid="empty-state"` - Empty state container
- `data-testid="welcome-heading"` - Welcome heading for new users
- `data-testid="create-first-poll-btn"` - Create first poll button
- `data-testid="poll-item-{id}"` - Individual poll list items
- `data-testid="poll-title-{id}"` - Poll titles in list
- `data-testid="poll-status-{id}"` - Poll status badges

### **Poll Creation Form**
- `data-testid="poll-title-input"` - Poll title input
- `data-testid="poll-description-input"` - Poll description textarea
- `data-testid="single-winner-radio"` - Single winner radio button
- `data-testid="multi-winner-radio"` - Multi winner radio button
- `data-testid="num-winners-input"` - Number of winners input
- `data-testid="candidate-name-{index}"` - Candidate name inputs
- `data-testid="add-candidate-btn"` - Add another candidate button
- `data-testid="cancel-poll-btn"` - Cancel button
- `data-testid="preview-poll-btn"` - Preview button
- `data-testid="create-poll-submit-btn"` - Create poll submit button

### **Voting Interface**
- `data-testid="voting-interface"` - Main voting container
- `data-testid="poll-header"` - Poll information section
- `data-testid="voting-instructions"` - Instructions section
- `data-testid="ranked-candidates-zone"` - Ranked candidates drop zone
- `data-testid="unranked-candidates-zone"` - Available candidates zone
- `data-testid="ranked-candidate-{id}"` - Individual ranked candidates
- `data-testid="unranked-candidate-{id}"` - Individual unranked candidates
- `data-testid="rank-candidate-btn-{id}"` - Rank candidate buttons
- `data-testid="unrank-candidate-btn-{id}"` - Unrank candidate buttons
- `data-testid="submit-ballot-btn"` - Submit ballot button
- `data-testid="voting-receipt"` - Voting receipt display

### **Poll Management (Voter Statistics)**
- `data-testid="voters-stats-card"` - Main voter statistics card
- `data-testid="overview-tab"` - Overview tab button
- `data-testid="voters-tab"` - Voters tab button
- `data-testid="results-tab"` - Results tab button
- `data-testid="voters-total-stat"` - Detailed total voters stat
- `data-testid="voters-voted-stat"` - Detailed voted count stat
- `data-testid="voters-pending-stat"` - Detailed pending count stat
- `data-testid="voter-email-input"` - Add voter email input
- `data-testid="add-voter-btn"` - Add voter button

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
npx playwright test homepage.test.ts
npx playwright test voting-interface.test.ts
npx playwright test complete-workflow.test.ts

# Run specific test
npx playwright test --grep "should complete full poll lifecycle workflow"
```

### Via Makefile
```bash
# Run E2E tests (from project root)
make test-e2e

# Run all tests including backend
make test
```

## Test Strategy

### Authentication Tests ✅
- Registration with valid data
- Login with existing credentials  
- Form validation errors
- Route protection (redirects)
- Logout functionality
- Invalid credentials handling

### Homepage & Landing Page Tests ✅
- Hero section content and navigation
- Features section display
- Call-to-action functionality
- Authentication-aware UI states
- Responsive design validation
- Cross-page navigation flows

### Poll Management Tests ✅
- Empty state for new users
- Single-winner and multi-winner poll creation
- Form validation (title, candidates, poll type)
- Candidate management (CRUD operations)
- Poll preview functionality
- Enhanced form features (cancel, preview, error handling)
- Dashboard display and navigation
- Date formatting verification
- Data persistence after refresh

### Voting Interface Tests ✅
- Complete voting workflow
- Drag-and-drop ranking functionality
- Candidate ranking and unranking
- Ballot submission and validation
- Voting receipt verification
- Double-voting prevention
- Error handling (invalid tokens, loading states)
- Mobile compatibility
- Empty state handling

### Voter Management Tests ✅
- Adding voters and statistics tracking
- Voter status updates (invited/voted/pending)
- Statistics display validation
- Bug fix regression testing
- Cross-tab statistics consistency

### Complete Workflow Tests ✅
- End-to-end poll lifecycle (creation → voting → results)
- Multi-user voting simulation
- Cross-session persistence
- Results verification
- Creator and voter perspective testing
- Authentication state management

### Integration Tests ✅
- Complete user journey (20+ steps)
- Multiple poll creation
- Cross-session persistence
- Error recovery workflows
- Authentication state management

## Test Data

Tests use unique identifiers (timestamps + random IDs) to avoid conflicts:
- Email: `test-${timestamp}-${randomId}@example.com`
- Poll titles: `Test Poll ${timestamp}-${randomId}`

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

## Coverage Summary

### ✅ **Comprehensive E2E Test Coverage (60+ Tests)**

**Authentication & Navigation** (6 tests):
- ✅ User registration with form validation
- ✅ User login with existing credentials
- ✅ Invalid credentials error handling
- ✅ Route protection and authentication redirects
- ✅ Logout functionality and session cleanup
- ✅ Navigation between pages

**Homepage & Landing Page** (11 tests):
- ✅ Hero section display and navigation
- ✅ Features section with all 6 features
- ✅ How it works section
- ✅ Authentication-aware UI states
- ✅ Call-to-action flows
- ✅ Responsive design (mobile/desktop)
- ✅ Cross-page navigation

**Poll Creation & Management** (15+ tests):
- ✅ Empty state display for new users
- ✅ Single-winner and multi-winner poll creation
- ✅ Form validation (title, candidates, poll type)
- ✅ Candidate management (add, remove, reorder)
- ✅ Enhanced form features (cancel, preview, error handling)
- ✅ Poll preview functionality
- ✅ Dashboard display and management
- ✅ Data persistence and state management

**Voting Interface** (12 tests):
- ✅ Complete voting interface display
- ✅ Drag-and-drop ranking functionality
- ✅ Rank/unrank candidate interactions
- ✅ Ballot submission and validation
- ✅ Voting receipt verification
- ✅ Double-voting prevention
- ✅ Error handling (invalid tokens, loading states)
- ✅ Mobile viewport compatibility
- ✅ Empty state handling

**Voter Management** (8+ tests):
- ✅ Adding voters and statistics tracking
- ✅ Voter status updates (invited/voted/pending)
- ✅ Statistics display validation
- ✅ Bug fix regression testing
- ✅ Cross-tab statistics consistency

**Complete Workflows** (6+ tests):
- ✅ Full poll lifecycle (creation → voting → results)
- ✅ Multi-user voting simulation
- ✅ Cross-session persistence testing
- ✅ Results verification after voting
- ✅ Error handling throughout workflow
- ✅ Authentication state management

**Integration Tests** (2 tests):
- ✅ Complete 20-step user journey
- ✅ Multiple poll creation with persistence

### **Total Coverage**: 60+ comprehensive E2E tests covering all user flows

## Benefits

The comprehensive E2E test suite provides:

- **Reliability**: Tests won't break when CSS classes or text content changes
- **Maintainability**: Clear, semantic test IDs that express intent
- **Team Consistency**: Standardized approach across all interactive elements
- **Performance**: Faster test execution with precise element targeting
- **Documentation**: Test IDs serve as living documentation of UI components
- **Confidence**: Complete user journey validation from registration to results
- **Regression Prevention**: Automated detection of bugs and regressions
- **CI/CD Ready**: Designed for automated testing in deployment pipelines

These tests complement the unit tests and provide confidence that the complete application works as expected from a user's perspective. 