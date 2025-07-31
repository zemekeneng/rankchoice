import { test, expect } from '@playwright/test';

test.describe('Poll Creation and Management', () => {
  const testUser = {
    email: `polls-test-${Date.now()}@example.com`,
    password: 'Test123!',
    name: 'Polls Test User'
  };

  test.beforeEach(async ({ page }) => {
    // Register and login for each test
    await page.goto('/register');
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');
    
    // Should be on dashboard now
    await expect(page).toHaveURL('/dashboard');
  });

  test('should show empty state when no polls exist', async ({ page }) => {
    // Should show welcome message for new users
    await expect(page.locator('text=Welcome to RankChoice!')).toBeVisible();
    await expect(page.locator('text=Create your first')).toBeVisible();
    await expect(page.locator('button:has-text("Create Your First Poll")')).toBeVisible();
  });

  test('should navigate to poll creation form', async ({ page }) => {
    // Click create poll button
    await page.click('button:has-text("Create Poll")');
    
    // Should navigate to poll creation page
    await expect(page).toHaveURL('/polls/new');
    await expect(page.locator('h1:has-text("Create Poll")')).toBeVisible();
  });

  test('should create a single-winner poll successfully', async ({ page }) => {
    // Navigate to poll creation
    await page.click('button:has-text("Create Poll")');
    await expect(page).toHaveURL('/polls/new');

    // Fill poll basic information
    await page.fill('input[id="title"]', 'E2E Test Poll - Single Winner');
    await page.fill('textarea[id="description"]', 'This is a test poll created by E2E tests');

    // Ensure single winner is selected (should be default)
    await expect(page.locator('input[value="single_winner"]')).toBeChecked();

    // Fill candidate information
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('Option A');
    await candidateInputs.nth(1).fill('Option B');

    // Add a third candidate
    await page.click('button:has-text("Add Another Candidate")');
    await candidateInputs.nth(2).fill('Option C');

    // Submit the form
    await page.click('button[type="submit"]:has-text("Create Poll")');

    // Should redirect to dashboard with success message
    await expect(page).toHaveURL('/dashboard?created=true');
    await expect(page.locator('text=Poll Created Successfully!')).toBeVisible();

    // Verify poll appears in dashboard list
    await expect(page.locator('text=E2E Test Poll - Single Winner')).toBeVisible();
    await expect(page.locator('text=Single Winner')).toBeVisible();
  });

  test('should create a multi-winner poll successfully', async ({ page }) => {
    await page.click('button:has-text("Create Poll")');

    // Fill basic information
    await page.fill('input[id="title"]', 'E2E Test Poll - Multi Winner');
    await page.fill('textarea[id="description"]', 'Multi-winner test poll');

    // Select multi-winner option
    await page.click('input[value="multi_winner"]');
    await expect(page.locator('input[value="multi_winner"]')).toBeChecked();

    // Set number of winners
    await page.fill('input[id="numWinners"]', '2');

    // Fill candidates
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('Candidate 1');
    await candidateInputs.nth(1).fill('Candidate 2');
    
    // Add more candidates for multi-winner
    await page.click('button:has-text("Add Another Candidate")');
    await candidateInputs.nth(2).fill('Candidate 3');
    await page.click('button:has-text("Add Another Candidate")');
    await candidateInputs.nth(3).fill('Candidate 4');

    // Submit form
    await page.click('button[type="submit"]:has-text("Create Poll")');

    // Verify success
    await expect(page).toHaveURL('/dashboard?created=true');
    await expect(page.locator('text=E2E Test Poll - Multi Winner')).toBeVisible();
    await expect(page.locator('text=Multi Winner (2)')).toBeVisible();
  });

  test('should validate required fields', async ({ page }) => {
    await page.click('button:has-text("Create Poll")');

    // Try to submit empty form
    await page.click('button[type="submit"]:has-text("Create Poll")');

    // Should show validation errors
    await expect(page.locator('text=Poll title is required')).toBeVisible();

    // Fill title but leave candidates empty
    await page.fill('input[id="title"]', 'Test Poll');
    
    // Clear candidate names
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('');
    await candidateInputs.nth(1).fill('');

    await page.click('button[type="submit"]:has-text("Create Poll")');

    // Should show candidate validation error
    await expect(page.locator('text=At least 2 candidates are required')).toBeVisible();
  });

  test('should show poll preview before creation', async ({ page }) => {
    await page.click('button:has-text("Create Poll")');

    // Fill valid form data
    await page.fill('input[id="title"]', 'Preview Test Poll');
    await page.fill('textarea[id="description"]', 'Testing the preview functionality');
    
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('Preview Option A');
    await candidateInputs.nth(1).fill('Preview Option B');

    // Click preview button
    await page.click('button:has-text("Preview")');

    // Should show preview modal/section
    await expect(page.locator('text=Poll Preview')).toBeVisible();
    await expect(page.locator('text=Preview Test Poll')).toBeVisible();
    await expect(page.locator('text=Preview Option A')).toBeVisible();
    await expect(page.locator('text=Preview Option B')).toBeVisible();

    // Should be able to create from preview
    await page.click('button:has-text("Create Poll")');
    await expect(page).toHaveURL('/dashboard?created=true');
  });

  test('should handle candidate management (add, remove, reorder)', async ({ page }) => {
    await page.click('button:has-text("Create Poll")');

    // Fill basic info
    await page.fill('input[id="title"]', 'Candidate Management Test');

    // Start with 2 default candidates
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('First Candidate');
    await candidateInputs.nth(1).fill('Second Candidate');

    // Add a third candidate
    await page.click('button:has-text("Add Another Candidate")');
    await candidateInputs.nth(2).fill('Third Candidate');

    // Verify 3 candidates exist
    await expect(candidateInputs).toHaveCount(3);

    // Try to remove a candidate (should have remove buttons when >2 candidates)
    const removeButtons = page.locator('button[title="Remove candidate"]');
    if (await removeButtons.count() > 0) {
      await removeButtons.first().click();
      await expect(candidateInputs).toHaveCount(2);
    }

    // Test reordering with move up/down buttons
    const moveUpButtons = page.locator('button[title="Move up"]');
    const moveDownButtons = page.locator('button[title="Move down"]');
    
    // These buttons should exist
    await expect(moveUpButtons.first()).toBeVisible();
    await expect(moveDownButtons.first()).toBeVisible();
  });

  test('should display correct date format in dashboard', async ({ page }) => {
    // Create a poll first
    await page.click('button:has-text("Create Poll")');
    await page.fill('input[id="title"]', 'Date Format Test Poll');
    
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('Option 1');
    await candidateInputs.nth(1).fill('Option 2');
    
    await page.click('button[type="submit"]:has-text("Create Poll")');
    await expect(page).toHaveURL('/dashboard?created=true');

    // Verify the date is formatted correctly (not "Invalid Date")
    const datePattern = /Created \w{3} \d{1,2}, \d{4}/; // e.g., "Created Jan 31, 2025"
    await expect(page.locator('text=' + datePattern.source)).toBeVisible();
    
    // Ensure no "Invalid Date" text appears
    await expect(page.locator('text=Invalid Date')).not.toBeVisible();
  });

  test('should persist data after page refresh', async ({ page }) => {
    // Create a poll
    await page.click('button:has-text("Create Poll")');
    await page.fill('input[id="title"]', 'Persistence Test Poll');
    
    const candidateInputs = page.locator('input[placeholder="Candidate name"]');
    await candidateInputs.nth(0).fill('Persistent Option A');
    await candidateInputs.nth(1).fill('Persistent Option B');
    
    await page.click('button[type="submit"]:has-text("Create Poll")');
    await expect(page).toHaveURL('/dashboard?created=true');

    // Verify poll exists
    await expect(page.locator('text=Persistence Test Poll')).toBeVisible();

    // Refresh the page
    await page.reload();

    // Data should still be there (no more welcome screen)
    await expect(page.locator('text=Persistence Test Poll')).toBeVisible();
    await expect(page.locator('text=Welcome to RankChoice!')).not.toBeVisible();
  });
}); 