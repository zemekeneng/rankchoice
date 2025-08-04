# E2E Testing Specification - Master Document

## 🚨 CRITICAL RULES

### Test Success Criteria
- **ALL TESTS MUST PASS** - No exceptions
- If any test fails, the system is broken and must be fixed
- **NEVER** claim a test "passed" when it failed
- **NEVER** create redundant tests when existing tests fail
- Fix the underlying issue, don't create workarounds

### Test Organization
- Each flow should have **ONE** comprehensive test file
- No duplicate or overlapping test coverage
- Tests must be deterministic and reliable
- All tests use consistent `data-testid` attributes

## Required E2E Test Flows

### 1. Authentication Flow (`auth.test.ts`)
**Purpose**: Verify user registration, login, logout, and route protection

**Test Cases**:
- ✅ User registration with valid data
- ✅ User login with existing credentials
- ✅ Invalid credentials handling
- ✅ Route protection (unauthenticated users redirected)
- ✅ Logout functionality
- ✅ Navigation links visibility based on auth state

**Critical Requirements**:
- Registration MUST fill all 4 fields: `name`, `email`, `password`, `confirmPassword`
- All navigation must use `data-testid` attributes
- Authentication state must persist across page refreshes

### 2. Homepage/Landing Page (`homepage.test.ts`)
**Purpose**: Verify homepage content and navigation for different user states

**Test Cases**:
- ✅ Hero section display with correct content
- ✅ Features section with all 6 features
- ✅ "How it Works" section display
- ✅ Authentication-aware UI (different buttons for logged in/out users)
- ✅ Navigation flows from homepage buttons
- ✅ Responsive design validation
- ✅ Cross-page navigation consistency

**Critical Requirements**:
- All interactive elements must have `data-testid` attributes
- Content must be visible and correctly formatted
- Navigation must work for both authenticated and unauthenticated users

### 3. Poll Creation and Management (`polls.test.ts`)
**Purpose**: Verify poll creation, form validation, and dashboard display

**Test Cases**:
- ✅ Empty state display for new users
- ✅ Single-winner poll creation with validation
- ✅ Multi-winner poll creation with validation
- ✅ Form validation errors and handling
- ✅ Candidate management (add, remove, reorder)
- ✅ Poll preview functionality
- ✅ Dashboard display and data persistence
- ✅ Date formatting verification

**Critical Requirements**:
- Form validation must prevent invalid submissions
- All form fields must use `data-testid` attributes
- Created polls must appear correctly on dashboard
- Data must persist after page refresh

### 4. Voter Management (`voters.test.ts`)
**Purpose**: Verify voter invitation, statistics, and management features

**Test Cases**:
- ✅ Adding voters to polls
- ✅ Voter statistics display (total, voted, pending)
- ✅ Voting token generation and links
- ✅ Voter status tracking
- ✅ Statistics accuracy across different states

**Critical Requirements**:
- Statistics must be accurate and update in real-time
- Voting tokens must be generated correctly
- All voter management UI must use `data-testid` attributes

### 5. Voting Interface (`voting-interface.test.ts`)
**Purpose**: Verify the complete voting experience and ballot submission

**Test Cases**:
- ✅ Voting interface display and poll information
- ✅ Drag-and-drop candidate ranking
- ✅ Rank/unrank candidate buttons
- ✅ Ballot submission and validation
- ✅ Voting receipt display
- ✅ Double-voting prevention
- ✅ Error handling for invalid tokens
- ✅ Mobile compatibility

**Critical Requirements**:
- Ranking functionality must work correctly
- Ballot submission must generate valid receipts
- Double-voting must be prevented
- All interactive elements must have `data-testid` attributes

### 6. Public/Anonymous Voting (`public-voting.test.ts`)
**Purpose**: Verify anonymous voting through public poll links

**Test Cases**:
- ✅ Public poll creation (with `is_public: true`)
- ✅ Public poll accessibility (no login required)
- ✅ Anonymous ballot submission
- ✅ Vote counting in creator dashboard
- ✅ Public poll data loading and display
- ✅ Anonymous vote receipt generation

**Critical Requirements**:
- Public polls must be accessible without authentication
- Anonymous votes must be counted in poll results
- Public voting UI must load poll data correctly
- Vote submission must work through public interface

### 7. Results Display (`results-display.test.ts`)
**Purpose**: Verify poll results calculation and display

**Test Cases**:
- ✅ Results display after voting
- ✅ RCV algorithm calculation accuracy
- ✅ Vote count and percentage display
- ✅ Winner determination
- ✅ Results visualization components
- ✅ Real-time results updates

**Critical Requirements**:
- Results must accurately reflect submitted votes
- RCV calculations must be mathematically correct
- Results UI must display clearly and correctly

### 8. Complete Workflow Integration (`complete-workflow.test.ts`)
**Purpose**: Verify end-to-end poll lifecycle from creation to results

**Test Cases**:
- ✅ Full user journey: Registration → Poll Creation → Voter Addition → Voting → Results
- ✅ Multi-voter simulation with different preferences
- ✅ Cross-session persistence
- ✅ Authentication state management
- ✅ Error handling throughout workflow
- ✅ Data consistency across all phases

**Critical Requirements**:
- Complete workflow must work without manual intervention
- All phases must integrate seamlessly
- Data must persist correctly throughout the entire process

## Prohibited Test Patterns

### ❌ DO NOT CREATE
- Multiple tests for the same functionality
- Debug tests that duplicate existing coverage
- "Simple" versions of existing comprehensive tests
- Tests that accept failure as normal behavior

### ❌ DO NOT DO
- Create new tests when existing tests fail - FIX THE FAILING TESTS
- Claim tests pass when they show errors or failures
- Skip assertions because "the API might work differently"
- Use placeholder comments like "TODO: Fix this test"

## Test Data Management

### Naming Conventions
- Use unique timestamps and random IDs: `test-${timestamp}-${randomId}@example.com`
- Ensure test data doesn't conflict between parallel runs
- Clean up test data where possible

### Required Test IDs
All interactive elements must have `data-testid` attributes. See `README.md` for complete list.

## Test Execution Standards

### Local Testing
```bash
# All tests must pass locally before committing
npm run test:e2e

# Debug individual tests
npx playwright test [test-name].test.ts --headed
```

### CI/CD Requirements
- All E2E tests must pass in CI
- Any test failure blocks deployment
- Tests must be deterministic and not flaky

## Success Metrics

### Definition of Success
- **100% test pass rate** - No failures tolerated
- All user flows work end-to-end without manual intervention
- UI elements load correctly and are interactive
- Data persistence works across all scenarios
- Authentication and authorization work correctly

### Failure Response
1. **Identify root cause** - Don't create workaround tests
2. **Fix the underlying issue** - Code, API, or UI problem
3. **Verify fix with existing tests** - Don't create new redundant tests
4. **Document any changes** - Update this specification if needed

## Maintenance

### Regular Audits
- Review test coverage monthly
- Remove redundant or outdated tests
- Update test IDs when UI changes
- Validate test performance and reliability

### Test Updates
- Update tests when features change
- Maintain backward compatibility where possible
- Document breaking changes clearly
- Ensure all team members understand test requirements

---

**Remember**: E2E tests are the final validation that our application works for real users. If they fail, the application is broken. Fix the application, not the tests.