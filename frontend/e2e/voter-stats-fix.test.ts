import { test, expect } from '@playwright/test';

test.describe('Voter Statistics Bug Fix Validation', () => {
  // Generate unique test user for this specific test
  const timestamp = Date.now();
  const randomId = Math.random().toString(36).substring(2, 8);
  const testUser = {
    email: `stats-fix-test-${timestamp}-${randomId}@example.com`,
    password: 'Test123!',
    name: 'Stats Fix Test User'
  };

  let pollId: string;
  let pollTitle: string;

  test.beforeAll(async ({ browser }) => {
    // Set up a poll with voters for testing the statistics display
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
      // Register and create a poll
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

      pollTitle = `Stats Fix Test Poll ${timestamp}-${randomId}`;
      await page.fill('input[id="title"]', pollTitle);
      await page.fill('textarea[id="description"]', 'Test poll for validating voter statistics bug fix');

      const candidateInputs = page.locator('input[placeholder="Candidate name"]');
      await expect(candidateInputs.first()).toBeVisible();
      
      // Fill the default candidate fields (focus on the bug fix, not candidate management)
      await candidateInputs.nth(0).fill('Option A');
      await candidateInputs.nth(1).fill('Option B');

      await page.click('button:has-text("Create Poll")');
      
      // Wait for redirect back to dashboard with created=true
      await page.waitForURL(/\/dashboard/, { timeout: 15000 });
      
      // Find the created poll and navigate to it
      await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
      await page.click(`text=${pollTitle}`);
      
      // Now we should be on the poll management page
      await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
      
      const url = page.url();
      pollId = url.split('/polls/')[1];

      // Add two voters
      await page.click('button:has-text("Voters")');
      
      // Add first voter
      await page.fill('input[type="email"]', 'voter1@statstest.com');
      await page.click('button:has-text("Add Voter")');
      await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('1');
      
      // Add second voter
      await page.fill('input[type="email"]', 'voter2@statstest.com');
      await page.click('button:has-text("Add Voter")');
      await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('2');

      // Get the first voter's voting URL and have them vote
      const openButtons = page.locator('a:has-text("Open")');
      const firstVotingUrl = await openButtons.nth(0).getAttribute('href');

      // Navigate to voting page and submit a vote
      await page.goto(firstVotingUrl!);
      await expect(page.locator(`h1:has-text("${pollTitle}")`)).toBeVisible();
      await page.click('text=Option A');
      await page.click('button:has-text("Submit Ballot")');
      await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    } finally {
      await context.close();
    }
  });

  test('should display correct voter statistics format - Bug Fix Validation', async ({ page }) => {
    // Login to access the poll
    await page.goto('/login');
    await expect(page.locator('h2:has-text("Sign in to your account")')).toBeVisible();
    
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');
    
    await expect(page).toHaveURL('/dashboard', { timeout: 10000 });

    // Navigate to the poll
    await page.goto(`/polls/${pollId}`);
    await expect(page.locator(`h1:has-text("${pollTitle}")`)).toBeVisible();

    // *** THIS IS THE KEY TEST FOR OUR BUG FIX ***
    // Verify the voter stats card shows the correct format: "2 (1 voted)"
    // Before the fix: it would show "2 (voted)" or "2 (undefined voted)"
    // After the fix: it should show "2 (1 voted)"
    
    const voterStatsCard = page.locator('.bg-white').filter({ hasText: 'Voters' });
    await expect(voterStatsCard.locator('.text-lg')).toContainText('2');
    
    // This is the critical assertion - the format should be "(1 voted)" not "(voted)"
    const votedText = voterStatsCard.locator('.text-sm');
    await expect(votedText).toContainText('(1 voted)');
    
    // Verify it doesn't show the broken format
    await expect(votedText).not.toContainText('(voted)'); // No number
    await expect(votedText).not.toContainText('(undefined voted)'); // Undefined value
    await expect(votedText).not.toContainText('(null voted)'); // Null value

    // Navigate to voters tab and verify detailed stats
    await page.click('button:has-text("Voters")');
    
    // Verify all voter stats show correct numbers (not undefined/null)
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('2');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('1');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('1');
    
    // Verify none of the stats show undefined, null, or NaN
    const statValues = page.locator('dt + dd');
    for (let i = 0; i < await statValues.count(); i++) {
      const text = await statValues.nth(i).textContent();
      expect(text).not.toContain('undefined');
      expect(text).not.toContain('null');
      expect(text).not.toContain('NaN');
    }

    // Test the tab badge shows correct count
    const votersTabBadge = page.locator('button:has-text("Voters") .bg-gray-100');
    await expect(votersTabBadge).toContainText('2');
    await expect(votersTabBadge).not.toContainText('undefined');
    await expect(votersTabBadge).not.toContainText('null');
  });

  test('should handle edge cases correctly - Zero voters', async ({ page }) => {
    // Login and create a new poll with zero voters
    await page.goto('/login');
    await expect(page.locator('h2:has-text("Sign in to your account")')).toBeVisible();
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/dashboard', { timeout: 10000 });

    // Create a new poll for zero voter testing
    await page.click('button:has-text("Create Poll")');
    await expect(page).toHaveURL('/polls/new', { timeout: 10000 });

    const zeroPollTitle = `Zero Voters Test ${timestamp}-${randomId}`;
    await page.fill('input[id="title"]', zeroPollTitle);
    await page.fill('textarea[id="description"]', 'Test poll with zero voters');

    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await expect(candidateInputs.first()).toBeVisible();
    
    await candidateInputs.nth(0).fill('Alpha');
    await candidateInputs.nth(1).fill('Beta');

    await page.click('button:has-text("Create Poll")');
    
    // Wait for redirect back to dashboard  
    await page.waitForURL(/\/dashboard/, { timeout: 15000 });
    
    // Find the created poll and navigate to it
    await expect(page.locator(`text=${zeroPollTitle}`)).toBeVisible();
    await page.click(`text=${zeroPollTitle}`);
    
    // Now we should be on the poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });

    // Verify zero voter stats display correctly
    const voterStatsCard = page.locator('.bg-white').filter({ hasText: 'Voters' });
    await expect(voterStatsCard.locator('.text-lg')).toContainText('0');
    
    // With zero voters, the vote count should not be shown at all
    const votedText = voterStatsCard.locator('.text-sm');
    await expect(votedText).not.toBeVisible();

    // Navigate to voters tab and verify zero stats
    await page.click('button:has-text("Voters")');
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('0');

    // Verify empty state is shown
    await expect(page.locator('text=No voters yet')).toBeVisible();
  });
}); 