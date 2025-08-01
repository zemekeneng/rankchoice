import { test, expect } from '@playwright/test';

test.describe('Voter Management and Statistics', () => {
  // Generate unique test user for each test run
  const timestamp = Date.now();
  const randomId = Math.random().toString(36).substring(2, 8);
  const testUser = {
    email: `voters-test-${timestamp}-${randomId}@example.com`,
    password: 'Test123!',
    name: 'Voters Test User'
  };

  let pollId: string;
  let pollTitle: string;

  test.beforeEach(async ({ page }) => {
    // Register and login for each test
    await page.goto('/register');
    
    // Wait for the page to load completely
    await expect(page.locator('h1:has-text("Register")')).toBeVisible();
    
    // Fill registration form
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[name="password"]', testUser.password);
    await page.fill('input[name="confirmPassword"]', testUser.password);
    
    // Submit and wait for navigation
    await page.click('button[type="submit"]');
    
    // Wait for successful registration and redirect to dashboard
    await expect(page).toHaveURL('/dashboard', { timeout: 10000 });

    // Create a test poll first
    await page.click('button:has-text("Create Poll")');
    await expect(page).toHaveURL('/polls/new', { timeout: 10000 });

    // Fill poll basic information
    pollTitle = `Voter Test Poll ${timestamp}-${randomId}`;
    await page.fill('input[id="title"]', pollTitle);
    await page.fill('textarea[id="description"]', 'Test poll for voter management');

    // Wait for candidate inputs to be available
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await expect(candidateInputs.first()).toBeVisible();
    
    // Fill the default candidate fields  
    await candidateInputs.nth(0).fill('Candidate Alpha');
    await candidateInputs.nth(1).fill('Candidate Beta');

    // Create the poll and wait for creation
    await page.click('button:has-text("Create Poll")');
    
    // Wait for redirect back to dashboard with created=true
    await page.waitForURL(/\/dashboard/, { timeout: 15000 });
    
    // Find the created poll and navigate to it
    await expect(page.locator(`text=${pollTitle}`)).toBeVisible();
    await page.click(`text=${pollTitle}`);
    
    // Now we should be on the poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
    
    // Extract poll ID from URL
    const url = page.url();
    pollId = url.split('/polls/')[1];
    
    // Verify we're on the poll management page
    await expect(page.locator(`h1:has-text("${pollTitle}")`)).toBeVisible({ timeout: 10000 });
  });

  test('should show correct initial voter statistics (0 voters)', async ({ page }) => {
    // Check the top stats cards show 0 voters initially
    const voterStatsCard = page.locator('.bg-white').filter({ hasText: 'Voters' });
    await expect(voterStatsCard.locator('.text-lg')).toContainText('0');
    
    // Check that there's no vote count shown when no voters exist
    await expect(voterStatsCard.locator('.text-sm')).not.toBeVisible();

    // Navigate to voters tab
    await page.click('button:has-text("Voters")');
    
    // Check voter overview shows all zeros
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('0');

    // Should show empty state
    await expect(page.locator('text=No voters yet')).toBeVisible();
  });

  test('should add voters and update statistics correctly', async ({ page }) => {
    // Navigate to voters tab
    await page.click('button:has-text("Voters")');
    
    // Add first voter with email
    await page.fill('input[type="email"]', 'voter1@example.com');
    await page.click('button:has-text("Add Voter")');
    
    // Wait for voter to be added and stats to update
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('1');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('1');

    // Add second voter without email (anonymous)
    await page.fill('input[type="email"]', '');
    await page.click('button:has-text("Add Voter")');
    
    // Wait for stats to update
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('2');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('2');

    // Add third voter with email
    await page.fill('input[type="email"]', 'voter3@example.com');
    await page.click('button:has-text("Add Voter")');
    
    // Final stats check
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('3');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('0');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('3');

    // Check that voters are listed
    await expect(page.locator('text=Voters (3)')).toBeVisible();
    await expect(page.locator('text=voter1@example.com')).toBeVisible();
    await expect(page.locator('text=voter3@example.com')).toBeVisible();
    
    // Check anonymous voter is shown with token suffix
    await expect(page.locator('[class*="text-sm font-medium"]:has-text("Voter")')).toHaveCount(1);
  });

  test('should show correct overview stats after adding voters', async ({ page }) => {
    // Add two voters first via voters tab
    await page.click('button:has-text("Voters")');
    
    // Add voters
    await page.fill('input[type="email"]', 'overview-test1@example.com');
    await page.click('button:has-text("Add Voter")');
    
    await page.fill('input[type="email"]', 'overview-test2@example.com');
    await page.click('button:has-text("Add Voter")');

    // Go back to overview tab
    await page.click('button:has-text("Overview")');
    
    // Check the main stats cards show correct numbers
    const voterStatsCard = page.locator('.bg-white').filter({ hasText: 'Voters' });
    await expect(voterStatsCard.locator('.text-lg')).toContainText('2');
    await expect(voterStatsCard.locator('.text-sm')).toContainText('(0 voted)');

    // Check candidates count
    const candidatesCard = page.locator('.bg-white').filter({ hasText: 'Candidates' });
    await expect(candidatesCard.locator('.text-lg')).toContainText('3');

    // Check total votes (should be 0)
    const votesCard = page.locator('.bg-white').filter({ hasText: 'Total Votes' });
    await expect(votesCard.locator('.text-lg')).toContainText('0');
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
    await expect(page.locator(`h1:has-text("${pollTitle}")`)).toBeVisible();
    await expect(page.locator('text=Rank the candidates')).toBeVisible();

    // Verify candidates are shown
    await expect(page.locator('text=Candidate Alpha')).toBeVisible();
    await expect(page.locator('text=Candidate Beta')).toBeVisible();
    await expect(page.locator('text=Candidate Gamma')).toBeVisible();

    // Drag and drop candidates to rank them (simulate ranking)
    // Note: This is a simplified ranking - in a real drag-and-drop we'd simulate the actual drag events
    // For now, we'll click the candidates in order to rank them
    await page.click('text=Candidate Beta'); // Rank 1
    await page.click('text=Candidate Alpha'); // Rank 2
    // Leave Candidate Gamma unranked

    // Submit the ballot
    await page.click('button:has-text("Submit Ballot")');
    
    // Should see confirmation page
    await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    // Go back to poll management page
    await page.goto(`/polls/${pollId}`);
    
    // Check updated stats in overview
    const voterStatsCard = page.locator('.bg-white').filter({ hasText: 'Voters' });
    await expect(voterStatsCard.locator('.text-lg')).toContainText('1');
    await expect(voterStatsCard.locator('.text-sm')).toContainText('(1 voted)');

    // Check total votes updated
    const votesCard = page.locator('.bg-white').filter({ hasText: 'Total Votes' });
    await expect(votesCard.locator('.text-lg')).toContainText('1');

    // Check voters tab statistics
    await page.click('button:has-text("Voters")');
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('1');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('1');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('0');

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

    // Have first voter vote
    await page.goto(firstVotingUrl!);
    await page.click('text=Candidate Alpha');
    await page.click('button:has-text("Submit Ballot")');
    await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    // Have second voter vote  
    await page.goto(secondVotingUrl!);
    await page.click('text=Candidate Beta');
    await page.click('button:has-text("Submit Ballot")');
    await expect(page.locator('text=Vote Submitted Successfully')).toBeVisible();

    // Return to poll management and check stats
    await page.goto(`/polls/${pollId}`);
    
    // Overview stats should show: 3 total, 2 voted
    const voterStatsCard = page.locator('.bg-white').filter({ hasText: 'Voters' });
    await expect(voterStatsCard.locator('.text-lg')).toContainText('3');
    await expect(voterStatsCard.locator('.text-sm')).toContainText('(2 voted)');

    // Total votes should be 2
    const votesCard = page.locator('.bg-white').filter({ hasText: 'Total Votes' });
    await expect(votesCard.locator('.text-lg')).toContainText('2');

    // Detailed voter stats
    await page.click('button:has-text("Voters")');
    await expect(page.locator('dt:has-text("Total Voters") + dd')).toContainText('3');
    await expect(page.locator('dt:has-text("Voted") + dd')).toContainText('2');
    await expect(page.locator('dt:has-text("Pending") + dd')).toContainText('1');

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