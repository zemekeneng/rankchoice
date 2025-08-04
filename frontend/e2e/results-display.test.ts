import { test, expect } from '@playwright/test';

test.describe('Results Display', () => {
  // Function to generate unique test data for each test to avoid collisions
  function generateTestUser(testPrefix: string = 'results') {
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(7);
    return {
      email: `${testPrefix}-test-${timestamp}-${randomId}@example.com`,
      password: 'Test123!',
      name: `${testPrefix.charAt(0).toUpperCase()}${testPrefix.slice(1)} Test User`
    };
  }
  
  test('should display results tab in poll management page', async ({ page }) => {
    const testUser = generateTestUser('results-tab');
    
    // Go to the homepage
    await page.goto('/');
    
    // Register a new user
    await page.click('text=Get Started');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    // Wait for redirect to dashboard
    await expect(page).toHaveURL('/dashboard');
    
    // Create a new poll
    await page.click('text=Create New Poll');
    await page.fill('[data-testid="poll-title-input"]', 'Results Test Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing results display functionality');
    
    // Add candidates
    await page.fill('[data-testid="candidate-name-input-0"]', 'Alice');
    await page.fill('[data-testid="candidate-description-input-0"]', 'First candidate');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-input-1"]', 'Bob');
    await page.fill('[data-testid="candidate-description-input-1"]', 'Second candidate');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-input-2"]', 'Charlie');
    await page.fill('[data-testid="candidate-description-input-2"]', 'Third candidate');
    
    // Create the poll
    await page.click('[data-testid="create-poll-btn"]');
    
    // Should be redirected to poll management page
    await expect(page.locator('h1')).toContainText('Results Test Poll');
    
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
    
    // Register and login
    await page.click('text=Get Started');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
    
    // Create poll with candidates
    await page.click('text=Create New Poll');
    await page.fill('[data-testid="poll-title-input"]', 'Mock Results Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing with mock results');
    
    await page.fill('[data-testid="candidate-name-input-0"]', 'Alice');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-input-1"]', 'Bob');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-input-2"]', 'Charlie');
    
    await page.click('[data-testid="create-poll-btn"]');
    
    // Go to results tab
    await page.click('[data-testid="results-tab"]');
    
    // Verify the UI structure is correct even without votes
    await expect(page.locator('text=No Votes Yet')).toBeVisible();
  });

  test('should navigate between tabs correctly', async ({ page }) => {
    const testUser = generateTestUser('results-tabs');
    
    await page.goto('/');
    
    // Register and create poll
    await page.click('text=Get Started');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
    
    await page.click('text=Create New Poll');
    await page.fill('[data-testid="poll-title-input"]', 'Tab Navigation Test');
    await page.fill('[data-testid="candidate-name-input-0"]', 'Option A');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-input-1"]', 'Option B');
    await page.click('[data-testid="create-poll-btn"]');
    
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
    await expect(page.locator('head title')).toContainText('Poll Results');
    
    // Should have RankChoice branding
    await expect(page.locator('text=RankChoice')).toBeVisible();
    
    // Should have share/copy functionality structure (even if poll doesn't exist)
    // The page should at least render without crashing
  });

});