import { test, expect } from '@playwright/test';

test.describe('Public Voting Flow', () => {
  test('should allow anonymous voting and display results in dashboard', async ({ page, context }) => {
    // Generate unique test data
    const testUser = {
      email: `publictest${Date.now()}@example.com`,
      password: 'TestPass123!', // Strong password with uppercase, lowercase, number, special char
      name: 'Public Test User'
    };

    const pollData = {
      title: 'Public Voting Test Poll',
      description: 'Testing anonymous public voting flow',
      candidates: ['Alice Johnson', 'Bob Smith', 'Carol Davis']
    };

    console.log('🚀 Starting public voting flow test...');

    // STEP 1: Register user and create public poll
    console.log('📝 Step 1: Register user and create poll');
    await page.goto('/register');
    
    // Fill all required registration fields (ORDER MATTERS!)
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password); // REQUIRED!
    
    console.log(`📝 Submitting registration for: ${testUser.email}`);
    await page.click('[data-testid="register-submit-btn"]');
    
    // Check for any error messages
    const errorLocator = page.locator('[data-testid="register-error"]');
    if (await errorLocator.isVisible({ timeout: 2000 })) {
      const errorText = await errorLocator.textContent();
      console.log(`❌ Registration error: ${errorText}`);
    }
    
    // Wait for dashboard with more debugging
    try {
      await expect(page).toHaveURL('/dashboard', { timeout: 10000 });
      console.log('✅ User registered and logged in');
    } catch (error) {
      console.log(`❌ Failed to reach dashboard. Current URL: ${page.url()}`);
      throw error;
    }

    // Create a new poll
    await page.click('[data-testid="create-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');

    // Fill poll details
    await page.fill('[data-testid="poll-title-input"]', pollData.title);
    await page.fill('[data-testid="poll-description-input"]', pollData.description);
    
    // Enable public voting
    await page.check('[data-testid="poll-public-checkbox"]');
    console.log('✅ Enabled public voting');

    // Fill existing candidate fields (form starts with 2 empty candidates)
    await page.fill('[data-testid="candidate-name-0"]', pollData.candidates[0]); // Alice Johnson
    await page.fill('[data-testid="candidate-name-1"]', pollData.candidates[1]); // Bob Smith
    
    // Add the third candidate
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-2"]', pollData.candidates[2]); // Carol Davis

    // Create the poll
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // After creation, it redirects to dashboard with created=true
    await expect(page).toHaveURL(/\/dashboard/);
    console.log('✅ Poll created and redirected to dashboard');
    
    // Find the newly created poll in the dashboard and click on it
    const pollTitleLocator = page.locator(`text=${pollData.title}`);
    await expect(pollTitleLocator).toBeVisible();
    await pollTitleLocator.click();
    
    // Now we should be on the poll management page
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/);
    const pollUrl = page.url();
    const pollId = pollUrl.split('/').pop();
    console.log(`✅ Navigated to poll management page with ID: ${pollId}`);

    // STEP 2: Get the public voting URL
    console.log('🔗 Step 2: Get public voting URL');
    
    // Construct the public voting URL directly (since it follows a predictable pattern)
    const publicVotingUrl = `/public/poll/${pollId}`;
    console.log(`✅ Public voting URL: ${publicVotingUrl}`);

    // STEP 3: Submit anonymous vote
    console.log('🗳️ Step 3: Submit anonymous vote');
    
    // Open new incognito context for anonymous voting
    const anonymousContext = await context.browser()?.newContext({
      // Use incognito mode to ensure no authentication
    });
    
    if (!anonymousContext) {
      throw new Error('Failed to create anonymous context');
    }

    const anonymousPage = await anonymousContext.newPage();
    
    // Navigate to public voting page
    await anonymousPage.goto(publicVotingUrl);
    
    // Check what's actually on the page (debugging)
    const pageTitle = await anonymousPage.locator('h1').textContent();
    console.log(`📄 Page title: ${pageTitle}`);
    const pageUrl = anonymousPage.url();
    console.log(`🌐 Current URL: ${pageUrl}`);
    
    // Look for poll information in a more flexible way
    const anonPollTitleLocator = anonymousPage.locator(`text=${pollData.title}`);
    const anonPollDescLocator = anonymousPage.locator(`text=${pollData.description}`);
    
    if (await anonPollTitleLocator.isVisible({ timeout: 2000 })) {
      console.log('✅ Public poll page loaded correctly');
    } else {
      console.log('⚠️ Poll title not found, checking if this is an error page or if public voting is not yet implemented');
      // Continue with the test anyway to see how far we get
    }

    // Start voting
    await anonymousPage.click('[data-testid="start-voting-btn"]');
    
    // Rank candidates (rank Alice first, Bob second)
    await anonymousPage.click(`[data-testid="rank-candidate-${pollData.candidates[0]}"]`);
    await anonymousPage.click(`[data-testid="rank-candidate-${pollData.candidates[1]}"]`);
    
    console.log('✅ Ranked candidates: Alice (1st), Bob (2nd)');

    // Submit the ballot
    await anonymousPage.click('[data-testid="submit-ballot-btn"]');
    
    // Wait for vote confirmation
    await expect(anonymousPage.locator('text=Vote Submitted')).toBeVisible({ timeout: 10000 });
    console.log('✅ Anonymous vote submitted successfully');
    
    await anonymousContext.close();

    // STEP 4: Check results in dashboard
    console.log('📊 Step 4: Check results in dashboard');
    
    // Go back to original page (logged in user)
    await page.goto(pollUrl);
    
    // Navigate to Results tab
    await page.click('[data-testid="results-tab"]');
    
    // Wait for results to load
    await page.waitForTimeout(2000); // Give backend time to process
    
    // Check that vote count is now 1
    const totalVotesLocator = page.locator('[data-testid="total-votes"]');
    await expect(totalVotesLocator).toBeVisible();
    
    const totalVotesText = await totalVotesLocator.textContent();
    console.log(`Total votes displayed: ${totalVotesText}`);
    
    // Verify vote count is 1
    await expect(totalVotesLocator).toContainText('1');
    
    // Check that Alice has votes (since we ranked her first)
    const aliceVotesLocator = page.locator(`[data-testid="candidate-votes-${pollData.candidates[0]}"]`);
    await expect(aliceVotesLocator).toBeVisible();
    
    const aliceVotes = await aliceVotesLocator.textContent();
    console.log(`Alice's vote count: ${aliceVotes}`);
    
    // Alice should have 1 vote (first choice)
    await expect(aliceVotesLocator).toContainText('1');
    
    console.log('🎉 SUCCESS: Anonymous vote appears in dashboard results!');

    // STEP 5: Submit another vote to verify multiple votes work
    console.log('🗳️ Step 5: Submit second anonymous vote');
    
    const secondContext = await context.browser()?.newContext();
    if (!secondContext) {
      throw new Error('Failed to create second anonymous context');
    }

    const secondPage = await secondContext.newPage();
    await secondPage.goto(publicVotingUrl);
    
    // Start voting
    await secondPage.click('[data-testid="start-voting-btn"]');
    
    // Rank differently: Carol first, Alice second
    await secondPage.click(`[data-testid="rank-candidate-${pollData.candidates[2]}"]`); // Carol
    await secondPage.click(`[data-testid="rank-candidate-${pollData.candidates[0]}"]`); // Alice
    
    // Submit the ballot
    await secondPage.click('[data-testid="submit-ballot-btn"]');
    await expect(secondPage.locator('text=Vote Submitted')).toBeVisible({ timeout: 10000 });
    
    console.log('✅ Second anonymous vote submitted');
    await secondContext.close();

    // Check updated results
    await page.reload();
    await page.click('[data-testid="results-tab"]');
    await page.waitForTimeout(2000);
    
    // Should now show 2 total votes
    await expect(totalVotesLocator).toContainText('2');
    console.log('✅ Total vote count updated to 2');
    
    // Both Alice and Carol should have votes
    const carolVotesLocator = page.locator(`[data-testid="candidate-votes-${pollData.candidates[2]}"]`);
    await expect(carolVotesLocator).toBeVisible();
    await expect(carolVotesLocator).toContainText('1'); // Carol got 1 first-choice vote
    
    console.log('🎉 COMPLETE SUCCESS: Multiple anonymous votes working correctly!');
  });
});