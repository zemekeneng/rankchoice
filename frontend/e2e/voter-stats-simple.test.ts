import { test, expect } from '@playwright/test';

test.describe('Voter Statistics Bug Fix - Simplified Validation', () => {
  // Generate unique test user
  const timestamp = Date.now();
  const randomId = Math.random().toString(36).substring(2, 8);
  const testUser = {
    email: `stats-simple-${timestamp}-${randomId}@example.com`,
    password: 'Test123!',
    name: 'Stats Simple Test User'
  };

  test('should display correct voter statistics format after adding voters', async ({ page }) => {
    // Register
    await page.goto('/register');
    await expect(page.locator('h2:has-text("Create your account")')).toBeVisible();
    
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[name="password"]', testUser.password);
    await page.fill('input[name="confirmPassword"]', testUser.password);
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/dashboard', { timeout: 10000 });

    // Create a test poll
    await page.click('button:has-text("Create Poll")');
    await expect(page).toHaveURL('/polls/new', { timeout: 10000 });

    const pollTitle = `Statistics Test Poll ${timestamp}-${randomId}`;
    await page.fill('input[id="title"]', pollTitle);
    await page.fill('textarea[id="description"]', 'Test poll for validating voter statistics bug fix');

    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await expect(candidateInputs.first()).toBeVisible();
    
    await candidateInputs.nth(0).fill('Option A');
    await candidateInputs.nth(1).fill('Option B');

    await page.click('button:has-text("Create Poll")');
    
    // Wait for redirect back to dashboard
    await page.waitForURL(/\/dashboard/, { timeout: 15000 });
    
    // Navigate to the poll
    await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
    await page.click(`text=${pollTitle}`);
    
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });

    // *** VERIFY INITIAL STATE (0 voters) ***
    // This tests our null coalescing fix
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('0');
    
    // With 0 voters, no vote count should be shown
    await expect(page.locator('[data-testid="voters-voted-count"]')).not.toBeVisible();

    // Navigate to voters tab and add voters
    await page.click('[data-testid="voters-tab"]');
    
    // Verify initial zero stats (tests our || 0 fix)
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('0');

    // *** CRITICAL TEST: Add voters and verify stats format ***
    
    // Add first voter
    await page.fill('[data-testid="voter-email-input"]', 'voter1@statstest.com');
    await page.click('[data-testid="add-voter-btn"]');
    
    // Verify stats update correctly (this tests the backend camelCase fix)
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('1');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('1');

    // Add second voter
    await page.fill('[data-testid="voter-email-input"]', 'voter2@statstest.com');
    await page.click('[data-testid="add-voter-btn"]');
    
    // Verify 2 voters, 0 voted (this is the key scenario from the bug report)
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('2');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('2');

    // Go back to overview to test the main stats card
    await page.click('[data-testid="overview-tab"]');
    
    // *** THIS IS THE CRITICAL BUG FIX TEST ***
    // Before fix: would show "2 (voted)" or "2 (undefined voted)" 
    // After fix: should show "2 (0 voted)"
    
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('2');
    await expect(page.locator('[data-testid="voters-voted-count"]')).toContainText('(0 voted)');
    
    // Verify it doesn't show the broken formats
    await expect(page.locator('[data-testid="voters-voted-count"]')).not.toContainText('(voted)'); // Missing number
    await expect(page.locator('[data-testid="voters-voted-count"]')).not.toContainText('(undefined voted)');
    await expect(page.locator('[data-testid="voters-voted-count"]')).not.toContainText('(null voted)');

    // Verify tab badge shows correct count
    await expect(page.locator('[data-testid="voters-tab-badge"]')).toContainText('2');
    await expect(page.locator('[data-testid="voters-tab-badge"]')).not.toContainText('undefined');

    // *** BONUS: Test adding a third voter ***
    await page.click('[data-testid="voters-tab"]');
    
    // Add anonymous voter (no email)
    await page.fill('[data-testid="voter-email-input"]', '');
    await page.click('[data-testid="add-voter-btn"]');
    
    // Should now show 3 total, 0 voted, 3 pending
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('3');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('3');

    // Check overview stats one more time
    await page.click('[data-testid="overview-tab"]');
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('3');
    await expect(page.locator('[data-testid="voters-voted-count"]')).toContainText('(0 voted)');
  });

  test('should handle voter statistics display edge cases', async ({ page }) => {
    // Test different scenarios for our bug fix
    
    // Create a second unique user for this test to avoid conflicts
    const edgeTestUser = {
      email: `edge-${timestamp}-${randomId}@example.com`,
      password: 'Test123!',
      name: 'Edge Test User'
    };
    
    // Register new user for this test
    await page.goto('/register');
    await expect(page.locator('h2:has-text("Create your account")')).toBeVisible();
    await page.fill('input[type="email"]', edgeTestUser.email);
    await page.fill('input[name="name"]', edgeTestUser.name);
    await page.fill('input[name="password"]', edgeTestUser.password);
    await page.fill('input[name="confirmPassword"]', edgeTestUser.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/dashboard', { timeout: 10000 });

    // Create a new poll for edge case testing
    await page.click('button:has-text("Create Poll")');
    await expect(page).toHaveURL('/polls/new', { timeout: 10000 });

    const edgePollTitle = `Edge Case Test ${timestamp}-${randomId}`;
    await page.fill('input[id="title"]', edgePollTitle);
    await page.fill('textarea[id="description"]', 'Test poll for edge cases');

    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await expect(candidateInputs.first()).toBeVisible();
    
    await candidateInputs.nth(0).fill('Choice X');
    await candidateInputs.nth(1).fill('Choice Y');

    await page.click('button:has-text("Create Poll")');
    await page.waitForURL(/\/dashboard/, { timeout: 15000 });
    
    await expect(page.locator(`text=${edgePollTitle}`)).toBeVisible();
    await page.click(`text=${edgePollTitle}`);
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });

    // Test rapid voter addition and statistics updates
    await page.click('[data-testid="voters-tab"]');
    
    const emails = ['edge1@test.com', 'edge2@test.com', 'edge3@test.com', 'edge4@test.com'];
    
    for (let i = 0; i < emails.length; i++) {
      await page.fill('[data-testid="voter-email-input"]', emails[i]);
      await page.click('[data-testid="add-voter-btn"]');
      
      // Verify stats update correctly after each addition
      const expectedTotal = i + 1;
      await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText(`${expectedTotal}`);
      await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
      await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText(`${expectedTotal}`);
    }

    // Final verification: 4 voters, 0 voted
    await page.click('[data-testid="overview-tab"]');
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('4');
    await expect(page.locator('[data-testid="voters-voted-count"]')).toContainText('(0 voted)');

    // Ensure no undefined/null values are displayed
    const allStatElements = page.locator('dd');
    for (let i = 0; i < await allStatElements.count(); i++) {
      const text = await allStatElements.nth(i).textContent();
      expect(text).not.toContain('undefined');
      expect(text).not.toContain('null');
      expect(text).not.toContain('NaN');
    }
  });
}); 