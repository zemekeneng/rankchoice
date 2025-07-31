import { test, expect } from '@playwright/test';

test.describe('Complete User Journey Integration', () => {
  test('should complete full user flow from registration to poll creation', async ({ page }) => {
    const timestamp = Date.now();
    const testUser = {
      email: `integration-${timestamp}@example.com`,
      password: 'Test123!',
      name: 'Integration Test User'
    };

    // Step 1: Start at home page
    await page.goto('/');
    await expect(page.locator('h1:has-text("RankChoice")')).toBeVisible();

    // Step 2: Navigate to registration
    await page.click('a[href="/register"]');
    await expect(page).toHaveURL('/register');

    // Step 3: Register new user
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');

    // Step 4: Should be redirected to dashboard
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator(`text=Welcome, ${testUser.name}`)).toBeVisible();

    // Step 5: Should see empty state for new user
    await expect(page.locator('text=Welcome to RankChoice!')).toBeVisible();
    await expect(page.locator('text=Create your first')).toBeVisible();

    // Step 6: Create first poll
    await page.click('button:has-text("Create Your First Poll")');
    await expect(page).toHaveURL('/polls/new');

    // Step 7: Fill poll creation form
    const pollTitle = `Integration Test Poll ${timestamp}`;
    await page.fill('input[id="title"]', pollTitle);
    await page.fill('textarea[id="description"]', 'This poll was created during integration testing');

    // Fill candidates
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('Integration Option A');
    await candidateInputs.nth(1).fill('Integration Option B');

    // Add description for candidates
    const candidateDescInputs = page.locator('textarea[placeholder="Brief description"]');
    await candidateDescInputs.nth(0).fill('First choice for integration testing');
    await candidateDescInputs.nth(1).fill('Second choice for integration testing');

    // Step 8: Preview the poll
    await page.click('button:has-text("Preview")');
    await expect(page.locator('text=Poll Preview')).toBeVisible();
    await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
    await expect(page.locator('text=Integration Option A')).toBeVisible();
    await expect(page.locator('text=Integration Option B')).toBeVisible();

    // Step 9: Create the poll from preview
    await page.click('button:has-text("Create Poll")');

    // Step 10: Should return to dashboard with success message
    await expect(page).toHaveURL('/dashboard?created=true');
    await expect(page.locator('text=Poll Created Successfully!')).toBeVisible();

    // Step 11: Verify poll appears in dashboard
    await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
    await expect(page.locator('text=Single Winner')).toBeVisible();
    await expect(page.locator('text=This poll was created during integration testing')).toBeVisible();

    // Step 12: Verify no more empty state
    await expect(page.locator('text=Welcome to RankChoice!')).not.toBeVisible();

    // Step 13: Verify date format is correct
    await expect(page.locator('text=Invalid Date')).not.toBeVisible();
    await expect(page.locator('text=Created')).toBeVisible();

    // Step 14: Create a second poll to verify multiple polls
    await page.click('button:has-text("Create Poll")');
    await page.fill('input[id="title"]', `Second Poll ${timestamp}`);
    
    const secondCandidateInputs = page.locator('input[placeholder="Candidate name"]');
    await secondCandidateInputs.nth(0).fill('Second Poll Option 1');
    await secondCandidateInputs.nth(1).fill('Second Poll Option 2');
    
    await page.click('button[type="submit"]:has-text("Create Poll")');

    // Step 15: Should now have 2 polls in dashboard
    await expect(page).toHaveURL('/dashboard?created=true');
    await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
    await expect(page.locator(`text=Second Poll ${timestamp}`)).toBeVisible();

    // Step 16: Test logout and login persistence
    await page.click('button:has-text("Logout")');
    await expect(page).toHaveURL('/');

    // Step 17: Login again
    await page.click('a[href="/login"]');
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');

    // Step 18: Should still see both polls after re-login
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
    await expect(page.locator(`text=Second Poll ${timestamp}`)).toBeVisible();

    // Step 19: Verify multi-winner poll creation
    await page.click('button:has-text("Create Poll")');
    await page.fill('input[id="title"]', `Multi-Winner Test ${timestamp}`);
    
    // Select multi-winner
    await page.click('input[value="multi_winner"]');
    await page.fill('input[id="numWinners"]', '2');
    
    // Add 4 candidates for multi-winner
    const multiCandidateInputs = page.locator('input[placeholder="Candidate name"]');
    await multiCandidateInputs.nth(0).fill('Multi Candidate 1');
    await multiCandidateInputs.nth(1).fill('Multi Candidate 2');
    await page.click('button:has-text("Add Another Candidate")');
    await multiCandidateInputs.nth(2).fill('Multi Candidate 3');
    await page.click('button:has-text("Add Another Candidate")');
    await multiCandidateInputs.nth(3).fill('Multi Candidate 4');
    
    await page.click('button[type="submit"]:has-text("Create Poll")');

    // Step 20: Verify multi-winner poll appears correctly
    await expect(page.locator(`text=Multi-Winner Test ${timestamp}`)).toBeVisible();
    await expect(page.locator('text=Multi Winner (2)')).toBeVisible();

    // Final verification: Should have 3 total polls
    const pollItems = page.locator('[data-testid="poll-item"], .poll-item, button:has-text("Created")').filter({ hasText: timestamp.toString() });
    await expect(pollItems).toHaveCount(3);
  });

  test('should handle form validation errors gracefully', async ({ page }) => {
    const testUser = {
      email: `validation-${Date.now()}@example.com`,
      password: 'Test123!',
      name: 'Validation Test User'
    };

    // Login first
    await page.goto('/register');
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');

    // Navigate to poll creation
    await page.click('button:has-text("Create Poll")');

    // Test empty form submission
    await page.click('button[type="submit"]:has-text("Create Poll")');
    await expect(page.locator('text=Poll title is required')).toBeVisible();

    // Test with title but no candidates
    await page.fill('input[id="title"]', 'Validation Test Poll');
    
    // Clear default candidates
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).clear();
    await candidateInputs.nth(1).clear();
    
    await page.click('button[type="submit"]:has-text("Create Poll")');
    await expect(page.locator('text=At least 2 candidates are required')).toBeVisible();

    // Fix form and ensure it works
    await candidateInputs.nth(0).fill('Valid Option A');
    await candidateInputs.nth(1).fill('Valid Option B');
    
    await page.click('button[type="submit"]:has-text("Create Poll")');
    await expect(page).toHaveURL('/dashboard?created=true');
    await expect(page.locator('text=Validation Test Poll')).toBeVisible();
  });
}); 