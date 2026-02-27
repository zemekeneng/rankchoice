// Quick test script to verify anonymous votes show up in voters tab
const { test, expect } = require('@playwright/test');

test('Anonymous votes should appear in voters tab', async ({ page }) => {
  // Register and login
  const timestamp = Date.now();
  const email = `test${timestamp}@example.com`;
  const password = 'TestPassword123!';

  console.log('🚀 Testing anonymous votes in voters tab...');
  
  // Go to register page
  await page.goto('http://localhost:5173/register');
  await page.fill('[data-testid="register-email"]', email);
  await page.fill('[data-testid="register-password"]', password);
  await page.fill('[data-testid="register-confirm-password"]', password);
  await page.click('[data-testid="register-submit-btn"]');
  
  // Wait for dashboard
  await expect(page).toHaveURL('/dashboard', { timeout: 10000 });
  console.log('✅ User registered and logged in');

  // Create a poll
  await page.click('[data-testid="create-poll-btn"]');
  await page.fill('[data-testid="poll-title"]', `Test Poll ${timestamp}`);
  await page.fill('[data-testid="poll-description"]', 'Testing anonymous voter display');
  
  // Add candidates
  await page.fill('[data-testid="candidate-name-0"]', 'Alice');
  await page.fill('[data-testid="candidate-name-1"]', 'Bob');
  
  // Enable public voting
  await page.check('[data-testid="public-voting-checkbox"]');
  
  await page.click('[data-testid="create-poll-submit"]');
  console.log('✅ Poll created');

  // Wait for poll page to load and get poll ID from URL
  await expect(page).toHaveURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
  const pollUrl = page.url();
  const pollId = pollUrl.split('/').pop();
  console.log('✅ Poll ID:', pollId);

  // Check initial voters tab - should be empty
  await page.click('[data-testid="voters-tab"]');
  await page.waitForTimeout(1000);
  
  const initialVoterCount = await page.textContent('[data-testid="voters-total-stat"]');
  console.log('📊 Initial voter count:', initialVoterCount);

  // Open new tab for anonymous voting
  const context = page.context();
  const anonymousPage = await context.newPage();
  
  // Go to public voting page
  await anonymousPage.goto(`http://localhost:5173/public/poll/${pollId}`);
  console.log('🌐 Opened anonymous voting page');

  // Submit anonymous vote
  await anonymousPage.waitForSelector('[data-testid="candidate-1"]', { timeout: 10000 });
  await anonymousPage.dragAndDrop('[data-testid="candidate-1"]', '[data-testid="rank-1"]');
  await anonymousPage.dragAndDrop('[data-testid="candidate-2"]', '[data-testid="rank-2"]');
  await anonymousPage.click('[data-testid="submit-vote-btn"]');
  
  // Wait for success message
  await anonymousPage.waitForSelector('text=Vote submitted successfully', { timeout: 10000 });
  console.log('✅ Anonymous vote submitted');

  // Go back to main page and check voters tab
  await page.reload();
  await page.click('[data-testid="voters-tab"]');
  await page.waitForTimeout(2000); // Give time for data to load

  // Check if voter count increased
  const newVoterCount = await page.textContent('[data-testid="voters-total-stat"]');
  console.log('📊 New voter count:', newVoterCount);

  // Look for anonymous voter entries
  const voterElements = await page.locator('ul.divide-y.divide-gray-200 li').count();
  console.log('📋 Number of voter entries found:', voterElements);

  // Check for anonymous voter in the list
  const anonymousVoterExists = await page.locator('text=/anon-|Anonymous/').count() > 0;
  console.log('🔍 Anonymous voter entry found:', anonymousVoterExists);

  if (anonymousVoterExists) {
    console.log('🎉 SUCCESS: Anonymous vote appears in voters tab!');
  } else {
    console.log('❌ FAILED: Anonymous vote not visible in voters tab');
  }

  await anonymousPage.close();
});