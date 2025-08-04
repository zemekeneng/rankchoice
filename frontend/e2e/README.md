# End-to-End Tests

This directory contains comprehensive Playwright E2E tests that verify the complete user flows of the RankChoice application.

**🚨 CRITICAL RULE: ALL TESTS MUST PASS - No exceptions. See `E2E_TESTING_SPECIFICATION.md` for complete requirements.**

## Test Files (Master Test Suite)

### `auth.test.ts`
**Authentication Flow** - Comprehensive user authentication testing:
- User registration with all required fields (name, email, password, confirmPassword)
- User login with existing credentials  
- Invalid credentials handling and error display
- Route protection (unauthenticated users redirected to login)
- Logout functionality and session cleanup
- Navigation links visibility based on authentication state

### `homepage.test.ts`
**Homepage/Landing Page** - Landing page functionality and navigation:
- Hero section display with correct content and messaging
- Features section with all 6 features properly displayed
- "How it Works" section display and content
- Authentication-aware UI (different buttons for logged in/out users)
- Navigation flows from homepage buttons (Get Started, Sign In, Dashboard)
- Responsive design validation on mobile viewports
- Cross-page navigation consistency

### `polls.test.ts`  
**Poll Creation and Management** - Complete poll lifecycle management:
- Empty state display for new users
- Single-winner poll creation with full validation
- Multi-winner poll creation with number of winners
- Form validation errors and proper error handling
- Candidate management (add, remove, reorder candidates)
- Poll preview functionality before creation
- Dashboard display and data persistence after refresh
- Date formatting verification (no "Invalid Date" errors)

### `voters.test.ts`
**Voter Management** - Voter invitation and statistics tracking:
- Adding voters to polls via email
- Voter statistics display (total voters, voted count, pending count)
- Voting token generation and shareable links
- Voter status tracking across different states
- Statistics accuracy and real-time updates
- Empty state and edge case handling

### `voting-interface.test.ts`
**Voting Interface** - Complete voting experience:
- Voting interface display and poll information presentation
- Drag-and-drop candidate ranking functionality
- Rank/unrank candidate buttons and interactions
- Ballot submission and validation processes
- Voting receipt display and verification codes
- Double-voting prevention and security
- Error handling for invalid tokens and edge cases
- Mobile viewport compatibility
- Loading states and empty state handling

### `public-voting.test.ts`
**Public/Anonymous Voting** - Anonymous voting through public poll links:
- Public poll creation with `is_public: true` flag
- Public poll accessibility without authentication requirement
- Anonymous ballot submission through public interface
- Vote counting integration with creator dashboard
- Public poll data loading and display verification
- Anonymous vote receipt generation and validation

### `results-display.test.ts`
**Results Display** - Poll results calculation and visualization:
- Results display accuracy after voting completion
- RCV (Ranked Choice Voting) algorithm calculation correctness
- Vote count and percentage display formatting
- Winner determination and announcement
- Results visualization components and charts
- Real-time results updates as votes are submitted

### `complete-workflow.test.ts`
**Complete Workflow Integration** - End-to-end poll lifecycle:
- Full user journey: Registration → Poll Creation → Voter Addition → Voting → Results
- Multi-voter simulation with different voting preferences
- Cross-session persistence and authentication state management
- Error handling and recovery throughout the entire workflow
- Data consistency verification across all phases
- Creator and voter perspective integration testing

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
- `data-testid="name-input"` - Registration name input ⚠️ **REQUIRED**
- `data-testid="register-email-input"` - Registration email input ⚠️ **REQUIRED**
- `data-testid="register-password-input"` - Registration password input ⚠️ **REQUIRED**
- `data-testid="confirm-password-input"` - Confirm password input ⚠️ **REQUIRED**
- `data-testid="register-submit-btn"` - Registration submit button
- `data-testid="register-error"` - Registration error message

> **🚨 CRITICAL:** Registration form requires ALL 4 fields: name, email, password, AND confirmPassword. Many E2E test failures are caused by missing the `confirm-password-input` field!

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

## Common E2E Test Patterns & Best Practices

### 🚨 **Registration Form - ALWAYS Use All 4 Fields**

**CORRECT Pattern:**
```typescript
await page.fill('[data-testid="name-input"]', testUser.name);
await page.fill('[data-testid="register-email-input"]', testUser.email);
await page.fill('[data-testid="register-password-input"]', testUser.password);
await page.fill('[data-testid="confirm-password-input"]', testUser.password); // DON'T FORGET!
await page.click('[data-testid="register-submit-btn"]');
```

**❌ WRONG Pattern (Missing confirm password):**
```typescript
await page.fill('[data-testid="register-email-input"]', testUser.email);
await page.fill('[data-testid="register-password-input"]', testUser.password);
// Missing confirm-password-input - WILL FAIL!
await page.click('[data-testid="register-submit-btn"]');
```

### 📝 **Public Voting Flow Pattern**
```typescript
// Create poll with public voting enabled
await page.check('[data-testid="poll-public-checkbox"]');

// Get public voting URL and test in incognito context
const anonymousContext = await context.browser()?.newContext();
const anonymousPage = await anonymousContext.newPage();
await anonymousPage.goto(publicVotingUrl);
```

### ⚡ **Form Field Order Matters**
Always fill form fields in the order they appear on the page to avoid race conditions and focus issues.

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

### ✅ **Comprehensive E2E Test Coverage (8 Test Files)**

**Authentication Flow** (`auth.test.ts`):
- ✅ User registration with form validation
- ✅ User login with existing credentials
- ✅ Invalid credentials error handling
- ✅ Route protection and authentication redirects
- ✅ Logout functionality and session cleanup
- ✅ Navigation between pages

**Homepage & Landing Page** (`homepage.test.ts`):
- ✅ Hero section display and navigation
- ✅ Features section with all 6 features
- ✅ How it works section
- ✅ Authentication-aware UI states
- ✅ Call-to-action flows
- ✅ Responsive design (mobile/desktop)
- ✅ Cross-page navigation

**Poll Creation & Management** (`polls.test.ts`):
- ✅ Empty state display for new users
- ✅ Single-winner and multi-winner poll creation
- ✅ Form validation (title, candidates, poll type)
- ✅ Candidate management (add, remove, reorder)
- ✅ Enhanced form features (cancel, preview, error handling)
- ✅ Poll preview functionality
- ✅ Dashboard display and management
- ✅ Data persistence and state management

**Voting Interface** (`voting-interface.test.ts`):
- ✅ Complete voting interface display
- ✅ Drag-and-drop ranking functionality
- ✅ Rank/unrank candidate interactions
- ✅ Ballot submission and validation
- ✅ Voting receipt verification
- ✅ Double-voting prevention
- ✅ Error handling (invalid tokens, loading states)
- ✅ Mobile viewport compatibility
- ✅ Empty state handling

**Voter Management** (`voters.test.ts`):
- ✅ Adding voters and statistics tracking
- ✅ Voter status updates (invited/voted/pending)
- ✅ Statistics display validation
- ✅ Bug fix regression testing
- ✅ Cross-tab statistics consistency

**Public/Anonymous Voting** (`public-voting.test.ts`):
- ✅ Public poll creation and accessibility
- ✅ Anonymous ballot submission through public interface
- ✅ Vote counting integration with creator dashboard
- ✅ Public poll data loading and verification

**Results Display** (`results-display.test.ts`):
- ✅ Results display accuracy and RCV calculation
- ✅ Vote count and percentage formatting
- ✅ Winner determination and visualization
- ✅ Real-time results updates

**Complete Workflow Integration** (`complete-workflow.test.ts`):
- ✅ Full poll lifecycle (creation → voting → results)
- ✅ Multi-user voting simulation
- ✅ Cross-session persistence testing
- ✅ Results verification after voting
- ✅ Error handling throughout workflow
- ✅ Authentication state management

**Complete Workflow Integration Tests**:
- ✅ Full end-to-end poll lifecycle with multi-user simulation
- ✅ Cross-session persistence and authentication management

### **Total Coverage**: 8 comprehensive test files covering all user flows

**Each test file covers ONE specific flow completely - no redundancy, no gaps.**

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