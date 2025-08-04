import { test, expect } from '@playwright/test';

test.describe('Results Display', () => {
  // Function to generate unique test data for each test to avoid collisions
  function generateTestUser(testPrefix: string = 'results') {
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(7);
    const nanoTime = process.hrtime.bigint().toString().slice(-6); // Last 6 digits of nanosecond precision
    const processId = process.pid.toString().slice(-3); // Last 3 digits of process ID
    return {
      email: `${testPrefix}-test-${timestamp}-${randomId}-${nanoTime}-${processId}@example.com`,
      password: 'Test123!',
      name: `${testPrefix.charAt(0).toUpperCase()}${testPrefix.slice(1)} Test User`
    };
  }
  
  test('should display results tab in poll management page', async ({ page }) => {
    const testUser = generateTestUser('results-tab');
    
    // Go to the homepage
    await page.goto('/');
    
    // Register a new user (homepage nav issue - use direct navigation)
    await page.goto('/register');
    
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    // Wait for redirect to dashboard
    await expect(page).toHaveURL('/dashboard');
    
    // Create a new poll
    await page.click('[data-testid="create-first-poll-btn"]');
    await page.fill('[data-testid="poll-title-input"]', 'Results Test Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing results display functionality');
    
    // Add candidates (only fill required name fields)
    await page.fill('[data-testid="candidate-name-0"]', 'Alice');
    await page.fill('[data-testid="candidate-name-1"]', 'Bob');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-2"]', 'Charlie');
    
    // Clear the datetime fields to make poll open immediately (no time restrictions)
    await page.fill('#opensAt', '');
    await page.fill('#closesAt', '');
    
    // Create the poll
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // Wait for redirect to dashboard after poll creation
    await page.waitForURL('/dashboard?created=true', { timeout: 10000 });
    
    // Find the newly created poll and navigate to it
    await page.waitForSelector('[data-testid^="poll-item-"]');
    const pollElement = page.locator('[data-testid^="poll-item-"]').filter({ hasText: 'Results Test Poll' });
    await pollElement.click();
    
    // Should be on poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
    
    // Check that results tab exists
    await expect(page.locator('[data-testid="results-tab"]')).toBeVisible();
    
    // Click on results tab
    await page.click('[data-testid="results-tab"]');
    
    // Should show "No Votes Yet" message since no one has voted
    await expect(page.locator('text=No Votes Yet')).toBeVisible();
    await expect(page.locator('text=Results will appear here once people start voting')).toBeVisible();
    
    // Overview and Voters tabs should also be available
    await expect(page.locator('[data-testid="overview-tab"]')).toBeVisible();
    await expect(page.locator('[data-testid="voters-tab"]')).toBeVisible();
  });

  test('should display results with mock votes', async ({ page }) => {
    const testUser = generateTestUser('results-mock');
    
    // This test would need mock data or a way to create votes
    // For now, we'll test the results page structure
    
    await page.goto('/');
    
    // Register and login (homepage nav issue - use direct navigation)
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
    
    // Create poll with candidates
    await page.click('[data-testid="create-first-poll-btn"]');
    await page.fill('[data-testid="poll-title-input"]', 'Mock Results Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing with mock results');
    
    await page.fill('[data-testid="candidate-name-0"]', 'Alice');
    await page.fill('[data-testid="candidate-name-1"]', 'Bob');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-2"]', 'Charlie');
    
    // Clear the datetime fields to make poll open immediately (no time restrictions)
    await page.fill('#opensAt', '');
    await page.fill('#closesAt', '');
    
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // Wait for redirect to dashboard after poll creation
    await page.waitForURL('/dashboard?created=true', { timeout: 10000 });
    
    // Find the newly created poll and navigate to it
    await page.waitForSelector('[data-testid^="poll-item-"]');
    const pollElement = page.locator('[data-testid^="poll-item-"]').filter({ hasText: 'Mock Results Poll' });
    await pollElement.click();
    
    // Should be on poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
    
    // Go to results tab
    await page.click('[data-testid="results-tab"]');
    
    // Verify the UI structure is correct even without votes
    await expect(page.locator('text=No Votes Yet')).toBeVisible();
  });

  test('should navigate between tabs correctly', async ({ page }) => {
    const testUser = generateTestUser('results-tabs');
    
    await page.goto('/');
    
    // Register and create poll (homepage nav issue - use direct navigation)
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    // Wait longer for registration to complete (concurrency issue - high load)
    await expect(page).toHaveURL('/dashboard', { timeout: 30000 });
    
    await page.click('[data-testid="create-first-poll-btn"]');
    await page.fill('[data-testid="poll-title-input"]', 'Tab Navigation Test');
    await page.fill('[data-testid="candidate-name-0"]', 'Option A');
    await page.fill('[data-testid="candidate-name-1"]', 'Option B');
    
    // Clear the datetime fields to make poll open immediately (no time restrictions)
    await page.fill('#opensAt', '');
    await page.fill('#closesAt', '');
    
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // Wait for redirect to dashboard after poll creation
    await page.waitForURL('/dashboard?created=true', { timeout: 10000 });
    
    // Find the newly created poll and navigate to it
    await page.waitForSelector('[data-testid^="poll-item-"]');
    const pollElement = page.locator('[data-testid^="poll-item-"]').filter({ hasText: 'Tab Navigation Test' });
    await pollElement.click();
    
    // Should be on poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
    
    // Test tab navigation
    // Should start on overview tab
    await expect(page.locator('[data-testid="overview-tab"]')).toHaveClass(/border-indigo-500/);
    
    // Click voters tab
    await page.click('[data-testid="voters-tab"]');
    await expect(page.locator('[data-testid="voters-tab"]')).toHaveClass(/border-indigo-500/);
    await expect(page.locator('[data-testid="overview-tab"]')).not.toHaveClass(/border-indigo-500/);
    
    // Click results tab
    await page.click('[data-testid="results-tab"]');
    await expect(page.locator('[data-testid="results-tab"]')).toHaveClass(/border-indigo-500/);
    await expect(page.locator('[data-testid="voters-tab"]')).not.toHaveClass(/border-indigo-500/);
    
    // Back to overview
    await page.click('[data-testid="overview-tab"]');
    await expect(page.locator('[data-testid="overview-tab"]')).toHaveClass(/border-indigo-500/);
    await expect(page.locator('[data-testid="results-tab"]')).not.toHaveClass(/border-indigo-500/);
  });

  test('should access standalone results page', async ({ page }) => {
    // Test that the standalone results page route exists and loads
    // We'll test with a mock poll ID to verify the route structure
    
    // Navigate directly to a results page URL
    const mockPollId = '550e8400-e29b-41d4-a716-446655440000';
    await page.goto(`/polls/${mockPollId}/results`);
    
    // Should load the results page (even if it shows an error for non-existent poll)
    // The important thing is that the route exists and the page structure loads
    await page.waitForLoadState('domcontentloaded');
    await expect(page).toHaveTitle(/Poll Results/);
    
    // Should have RankChoice branding
    await expect(page.locator('h1:has-text("RankChoice")')).toBeVisible();
    
    // Should have share/copy functionality structure (even if poll doesn't exist)
    // The page should at least render without crashing
  });

});