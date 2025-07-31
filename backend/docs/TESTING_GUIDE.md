# Testing Guide for RankChoice.app Backend

## Overview

This document outlines the comprehensive testing strategy for the RankChoice.app backend API. All components must be thoroughly tested before being marked as complete.

## Testing Requirements

### Authentication System Testing ✅ IMPLEMENTED

#### Unit Tests (`auth_unit_tests.rs`)
- ✅ **Password Hashing Tests**
  - Test password hashing produces different salts
  - Test password verification works correctly
  - Test wrong passwords are rejected

- ✅ **JWT Token Tests**
  - Test access token generation and verification
  - Test refresh token generation and verification
  - Test token expiration differences
  - Test invalid token handling

- ✅ **AuthService Business Logic Tests**
  - Test user registration service
  - Test user login service
  - Test duplicate user registration prevention
  - Test invalid credential handling
  - Test refresh token functionality

#### Integration Tests (`auth_integration_tests.rs`)
- ✅ **Registration Endpoint Tests**
  - Test successful user registration
  - Test duplicate email handling
  - Test invalid data validation
  - Test API response format

- ✅ **Login Endpoint Tests**
  - Test successful login
  - Test invalid credentials
  - Test non-existent user
  - Test API response format

- ✅ **Refresh Token Endpoint Tests**
  - Test successful token refresh
  - Test invalid refresh token
  - Test API response format

- ✅ **Error Handling Tests**
  - Test all error response formats
  - Test HTTP status codes
  - Test error message consistency

## Testing Strategy

### Test Categories

1. **Unit Tests** - Test individual functions and methods in isolation
2. **Integration Tests** - Test API endpoints with real database
3. **Database Tests** - Test data persistence and retrieval
4. **Security Tests** - Test authentication and authorization
5. **Error Handling Tests** - Test all error scenarios

### Test Database

- Uses SQLx testing framework with `#[sqlx::test]`
- Each test gets a fresh, isolated database instance
- Migrations are run automatically for each test
- Tests are run in parallel safely

### Test Fixtures

- Common test utilities in `tests/common/mod.rs`
- Test app creation helpers
- Mock data generators
- Database seeding utilities

## Testing Commands

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test auth_unit_tests

# Run only integration tests
cargo test --test auth_integration_tests

# Run tests with output
cargo test -- --nocapture

# Run tests in single thread (for debugging)
cargo test -- --test-threads=1
```

## Future Testing Requirements

### Polls API Testing (Pending Implementation)
When implementing the Polls API, the following tests must be created:

#### Unit Tests Required
- [ ] Poll creation validation
- [ ] Poll update logic
- [ ] Poll deletion logic
- [ ] Poll status management
- [ ] Poll ownership validation

#### Integration Tests Required
- [ ] `POST /api/polls` - Create poll endpoint
- [ ] `GET /api/polls` - List polls endpoint
- [ ] `GET /api/polls/:id` - Get poll details endpoint
- [ ] `PUT /api/polls/:id` - Update poll endpoint
- [ ] `DELETE /api/polls/:id` - Delete poll endpoint
- [ ] Authentication middleware testing
- [ ] Authorization testing (user can only access their polls)

### Candidates API Testing (Pending Implementation)
When implementing the Candidates API, the following tests must be created:

#### Unit Tests Required
- [ ] Candidate creation validation
- [ ] Candidate ordering logic
- [ ] Candidate deletion handling

#### Integration Tests Required
- [ ] `POST /api/polls/:id/candidates` - Add candidate
- [ ] `PUT /api/candidates/:id` - Update candidate
- [ ] `DELETE /api/candidates/:id` - Delete candidate
- [ ] `PUT /api/polls/:id/candidates/order` - Reorder candidates

### Voting API Testing (Pending Implementation)
When implementing the Voting API, the following tests must be created:

#### Unit Tests Required
- [ ] Ballot token generation and validation
- [ ] Vote ranking validation
- [ ] Duplicate vote prevention
- [ ] Vote counting logic

#### Integration Tests Required
- [ ] `GET /api/vote/:token` - Get ballot
- [ ] `POST /api/vote/:token` - Submit ballot
- [ ] `GET /api/vote/:token/receipt` - Get receipt
- [ ] Invalid token handling
- [ ] Already voted handling

### RCV Algorithm Testing (Pending Implementation)
When implementing the RCV tabulation system:

#### Unit Tests Required
- [ ] Single-winner RCV algorithm
- [ ] Multi-winner RCV algorithm (STV)
- [ ] Tie-breaking logic
- [ ] Edge case handling (all candidates eliminated, etc.)
- [ ] Vote transfer calculations

#### Integration Tests Required
- [ ] `GET /api/polls/:id/results` - Get results
- [ ] `GET /api/polls/:id/results/rounds` - Get round details
- [ ] Real-time result updates
- [ ] Result caching

## Test Coverage Requirements

- **Minimum Coverage**: 90% line coverage for all modules
- **Branch Coverage**: 85% branch coverage for business logic
- **Integration Coverage**: All API endpoints must have tests
- **Error Coverage**: All error paths must be tested

## Test Quality Standards

### Test Naming Convention
- Test function names should be descriptive: `test_register_with_duplicate_email`
- Test modules should be grouped by feature: `auth_tests`, `polls_tests`

### Test Structure
Each test should follow the Arrange-Act-Assert pattern:
```rust
#[sqlx::test]
async fn test_example(pool: PgPool) {
    // Arrange - Set up test data
    let auth_service = AuthService::new(pool);
    let request = CreateUserRequest { /* ... */ };
    
    // Act - Perform the action being tested
    let result = auth_service.register(request).await;
    
    // Assert - Verify the results
    assert!(result.is_ok());
    assert_eq!(result.unwrap().user.email, "test@example.com");
}
```

### Test Data
- Use realistic but obviously fake data
- Email addresses should use `@example.com` domain
- Passwords should be test-specific but realistic
- UUIDs should be generated fresh for each test

## Continuous Integration

Tests must pass in CI before any code can be merged:
- All tests must pass
- No test should be marked `#[ignore]` without justification
- Test execution time should be reasonable (< 2 minutes total)
- Tests should be reliable and not flaky

## Test Maintenance

- Update tests when APIs change
- Add new tests for new features
- Remove tests for deprecated features
- Keep test dependencies up to date
- Review test coverage regularly

## Security Testing Considerations

- Test all authentication and authorization scenarios
- Test input validation and sanitization
- Test SQL injection prevention
- Test XSS prevention
- Test rate limiting
- Test JWT token security

## Performance Testing

While not covered in unit tests, consider these for load testing:
- API response times under load
- Database performance with large datasets
- Concurrent user handling
- Memory usage patterns

## Completion Criteria

A feature is considered complete when:
1. All unit tests pass
2. All integration tests pass
3. Test coverage meets requirements
4. All error scenarios are tested
5. Security tests pass
6. Documentation is updated
7. Tests are added to CI pipeline 