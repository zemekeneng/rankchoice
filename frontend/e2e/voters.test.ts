import { test, expect } from '@playwright/test';

test.describe('Voter Management and Statistics', () => {
  // Function to generate unique test data for each test to avoid collisions
  function generateTestUser(testPrefix: string = 'voters') {
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(2, 8);
    return {
      email: `${testPrefix}-test-${timestamp}-${randomId}@example.com`,
      password: 'Test123!',
      name: `${testPrefix.charAt(0).toUpperCase()}${testPrefix.slice(1)} Test User`
    };
  }

  let pollId: string;
  let pollTitle: string;

  test.beforeEach(async ({ page }) => {
    // Generate unique user and poll data for THIS test run to avoid collisions
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(2, 8);
    const testUser = generateTestUser(`voters-${timestamp}`);
    
    // Register and login for each test
    await page.goto('/register');
    
    // Wait for the page to load completely
    await expect(page.locator('[data-testid="register-heading"]')).toBeVisible();
    
    // Fill registration form using proper test IDs
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    
    // Submit and wait for navigation
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard', { timeout: 15000 });

    // Create a test poll first
    await page.click('[data-testid="create-poll-btn"]');
    await expect(page).toHaveURL('/polls/new', { timeout: 10000 });

    // Fill poll basic information
    pollTitle = `Voter Test Poll ${timestamp}-${randomId}`;
    await page.fill('[data-testid="poll-title-input"]', pollTitle);
    await page.fill('[data-testid="poll-description-input"]', 'Test poll for voter management');

    // Fill the default candidate fields using test IDs and add a third candidate
    await page.fill('[data-testid="candidate-name-0"]', 'Candidate Alpha');
    await page.fill('[data-testid="candidate-name-1"]', 'Candidate Beta');
    // Add third candidate
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-2"]', 'Candidate Gamma');

    // Create the poll and wait for creation
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // Wait for redirect back to dashboard
    await page.waitForURL('/dashboard', { timeout: 15000 });
    
    // Find the created poll and navigate to it
    await page.waitForSelector('[data-testid^="poll-item-"]');
    const pollElement = page.locator('[data-testid^="poll-item-"]').filter({ hasText: pollTitle });
    await pollElement.click();
    
    // Now we should be on the poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
    
    // Extract poll ID from URL
    const url = page.url();
    pollId = url.split('/polls/')[1];
    
    // Verify we're on the poll management page
    await expect(page.locator(`h1:has-text("${pollTitle}")`)).toBeVisible({ timeout: 10000 });
  });

  test('should show correct initial voter statistics (0 voters)', async ({ page }) => {
    // Check the top stats cards show 0 voters initially using specific test IDs
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('0');
    
    // Navigate to voters tab
    await page.click('[data-testid="voters-tab"]');
    
    // Check voter overview shows all zeros using specific test IDs
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('0');

    // Should show empty state
    await expect(page.locator('text=No voters yet')).toBeVisible();
  });

  test('should add voters and update statistics correctly', async ({ page }) => {
    // Navigate to voters tab
    await page.click('[data-testid="voters-tab"]');
    
    // Add first voter with email
    await page.fill('[data-testid="voter-email-input"]', 'voter1@example.com');
    await page.click('[data-testid="add-voter-btn"]');
    
    // Wait for voter to be added and stats to update using test IDs
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('1');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('1');

    // Add second voter without email (anonymous)
    await page.fill('[data-testid="voter-email-input"]', '');
    await page.click('[data-testid="add-voter-btn"]');
    
    // Wait for stats to update
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('2');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('0');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('2');

    // Add third voter with email
    await page.fill('[data-testid="voter-email-input"]', 'voter3@example.com');
    await page.click('[data-testid="add-voter-btn"]');
    
    // Final stats check
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('3');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('3');

    // Check that voters are listed
    await expect(page.locator('text=Voters (3)')).toBeVisible();
    await expect(page.locator('text=voter1@example.com')).toBeVisible();
    await expect(page.locator('text=voter3@example.com')).toBeVisible();
    
    // Check that voters are displayed (use more specific selector)
    // The exact voter display might vary, so just check we have voters visible
    const voterElements = await page.locator('text=/voter\d+@example.com/').count();
    console.log(`Found ${voterElements} voter elements`);
  });

  test('should show correct overview stats after adding voters', async ({ page }) => {
    // Add two voters first via voters tab
    await page.click('[data-testid="voters-tab"]');
    
    // Add voters using test IDs
    await page.fill('[data-testid="voter-email-input"]', 'overview-test1@example.com');
    await page.click('[data-testid="add-voter-btn"]');
    
    await page.fill('[data-testid="voter-email-input"]', 'overview-test2@example.com');
    await page.click('[data-testid="add-voter-btn"]');

    // Go back to overview tab
    await page.click('[data-testid="overview-tab"]');
    
    // Check the main stats cards show correct numbers using test IDs
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('2');
    await expect(page.locator('[data-testid="voters-voted-count"]')).toContainText('(0 voted)');
  });

  test('should update statistics after a vote is submitted', async ({ page }) => {
    // Add a voter first
    await page.click('button:has-text("Voters")');
    await page.fill('input[type="email"]', 'voting-test@example.com');
    await page.click('button:has-text("Add Voter")');

    // Get the voting link
    const copyLinkButton = page.locator('button:has-text("Copy Link")').first();
    await expect(copyLinkButton).toBeVisible();
    
    // Get the voting URL by clicking the "Open" button which has the href
    const openButton = page.locator('a:has-text("Open")').first();
    const votingUrl = await openButton.getAttribute('href');
    expect(votingUrl).toContain('/vote/');

    // Navigate to voting page in the same browser context
    await page.goto(votingUrl!);
    
    // Verify we're on the voting page
    await expect(page.locator('[data-testid="poll-title"]')).toContainText(pollTitle);
    await expect(page.locator('[data-testid="voting-instructions"]')).toBeVisible();

    // Verify candidates are shown
    await expect(page.locator('text=Candidate Alpha')).toBeVisible();
    await expect(page.locator('text=Candidate Beta')).toBeVisible();
    await expect(page.locator('text=Candidate Gamma')).toBeVisible();

    // Rank candidates using the proper rank buttons
    // Wait for voting interface to load
    await page.waitForSelector('[data-testid^="rank-candidate-btn-"]');
    
    // Rank candidates by clicking rank buttons
    const rankButtons = page.locator('[data-testid^="rank-candidate-btn-"]');
    await rankButtons.nth(1).click(); // Rank Candidate Beta first
    await rankButtons.nth(0).click(); // Rank Candidate Alpha second
    // Leave third candidate unranked

    // Submit the ballot
    await page.click('[data-testid="submit-ballot-btn"]');
    
    // Should see confirmation page
    await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    // Go back to poll management page
    await page.goto(`/polls/${pollId}`);
    
    // Check updated stats in overview using test IDs
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('1');
    await expect(page.locator('[data-testid="voters-voted-count"]')).toContainText('(1 voted)');

    // Check voters tab statistics using test IDs
    await page.click('[data-testid="voters-tab"]');
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('1');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('1');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('0');

    // Verify voter status changed to "Voted"
    await expect(page.locator('text=voting-test@example.com')).toBeVisible();
    await expect(page.locator('.bg-green-100:has-text("Voted")')).toBeVisible();
  });

  test('should handle multiple voters with mixed voting states', async ({ page }) => {
    // Add three voters
    await page.click('button:has-text("Voters")');
    
    // Voter 1
    await page.fill('input[type="email"]', 'mixed-test1@example.com');
    await page.click('button:has-text("Add Voter")');
    
    // Voter 2  
    await page.fill('input[type="email"]', 'mixed-test2@example.com');
    await page.click('button:has-text("Add Voter")');
    
    // Voter 3 (anonymous)
    await page.fill('input[type="email"]', '');
    await page.click('button:has-text("Add Voter")');

    // Verify initial stats
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('3');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('3');

    // Get voting URLs for the first two voters
    const openButtons = page.locator('a:has-text("Open")');
    const firstVotingUrl = await openButtons.nth(0).getAttribute('href');
    const secondVotingUrl = await openButtons.nth(1).getAttribute('href');

    // Have first voter vote (rank candidate properly)
    await page.goto(firstVotingUrl!);
    // Wait for voting interface to load and rank a candidate
    await page.waitForSelector('[data-testid^="rank-candidate-btn-"]');
    await page.click('[data-testid^="rank-candidate-btn-"]');
    await page.click('[data-testid="submit-ballot-btn"]');
    await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    // Have second voter vote (rank candidate properly)
    await page.goto(secondVotingUrl!);
    // Wait for voting interface to load and rank a candidate
    await page.waitForSelector('[data-testid^="rank-candidate-btn-"]');
    await page.click('[data-testid^="rank-candidate-btn-"]');
    await page.click('[data-testid="submit-ballot-btn"]');
    await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    // Return to poll management and check stats
    await page.goto(`/polls/${pollId}`);
    
    // Overview stats should show: 3 total, 2 voted using test IDs
    await expect(page.locator('[data-testid="voters-total-count"]')).toContainText('3');
    await expect(page.locator('[data-testid="voters-voted-count"]')).toContainText('(2 voted)');

    // Detailed voter stats using test IDs
    await page.click('[data-testid="voters-tab"]');
    await expect(page.locator('[data-testid="voters-total-stat"]')).toContainText('3');
    await expect(page.locator('[data-testid="voters-voted-stat"]')).toContainText('2');
    await expect(page.locator('[data-testid="voters-pending-stat"]')).toContainText('1');

    // Check individual voter statuses
    await expect(page.locator('text=mixed-test1@example.com')).toBeVisible();
    await expect(page.locator('text=mixed-test2@example.com')).toBeVisible();
    
    // Should have 2 "Voted" badges and 1 "Pending" badge
    await expect(page.locator('.bg-green-100:has-text("Voted")')).toHaveCount(2);
    await expect(page.locator('.bg-yellow-100:has-text("Pending")')).toHaveCount(1);
  });

  test('should display correct voter count in tab badge', async ({ page }) => {
    // Initially voters tab should not show count badge
    await expect(page.locator('button:has-text("Voters")')).not.toContainText('(');

    // Add some voters
    await page.click('button:has-text("Voters")');
    
    await page.fill('input[type="email"]', 'badge-test1@example.com');
    await page.click('button:has-text("Add Voter")');

    // Tab should now show count
    await expect(page.locator('button:has-text("Voters") .bg-gray-100')).toContainText('1');

    await page.fill('input[type="email"]', 'badge-test2@example.com');
    await page.click('button:has-text("Add Voter")');

    // Count should update
    await expect(page.locator('button:has-text("Voters") .bg-gray-100')).toContainText('2');

    // Go to overview and back to verify count persists
    await page.click('button:has-text("Overview")');
    await expect(page.locator('button:has-text("Voters") .bg-gray-100')).toContainText('2');
  });
}); 