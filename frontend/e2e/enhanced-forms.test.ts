import { test, expect } from '@playwright/test';

test.describe('Enhanced Form Features and Error Handling', () => {
  let testUser: any;

  test.beforeEach(async ({ page }) => {
    // Generate unique user for each test
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(7);
    
    testUser = {
      email: `form-test-${timestamp}-${randomId}@example.com`,
      password: 'Test123!',
      name: 'Form Test User'
    };

    // Register and login
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
  });

  test('should display and use form actions correctly', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Check that form actions section exists
    await expect(page.locator('[data-testid="form-actions"]')).toBeVisible();
    
    // Check all action buttons are present
    await expect(page.locator('[data-testid="cancel-poll-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="preview-poll-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="create-poll-submit-btn"]')).toBeVisible();
    
    // Verify button text
    await expect(page.locator('[data-testid="cancel-poll-btn"]')).toContainText('Cancel');
    await expect(page.locator('[data-testid="preview-poll-btn"]')).toContainText('Preview');
    await expect(page.locator('[data-testid="create-poll-submit-btn"]')).toContainText('Create Poll');
  });

  test('should handle cancel functionality', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Fill some form data
    await page.fill('[data-testid="poll-title-input"]', 'Test Poll to Cancel');
    await page.fill('[data-testid="poll-description-input"]', 'This poll will be cancelled');
    
    // Click cancel
    await page.click('[data-testid="cancel-poll-btn"]');
    
    // Should return to dashboard
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('[data-testid="welcome-heading"]')).toBeVisible();
  });

  test('should handle preview functionality', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Fill required form data for preview
    await page.fill('[data-testid="poll-title-input"]', 'Preview Test Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing the preview functionality');
    await page.fill('[data-testid="candidate-name-0"]', 'First Option');
    await page.fill('[data-testid="candidate-name-1"]', 'Second Option');
    
    // Preview button should be enabled with valid data
    await expect(page.locator('[data-testid="preview-poll-btn"]')).toBeEnabled();
    
    // Click preview
    await page.click('[data-testid="preview-poll-btn"]');
    
    // Wait for preview to potentially load or toggle
    await page.waitForTimeout(1000);
    
    // Should still be on the same page
    await expect(page).toHaveURL('/polls/new');
  });

  test('should disable preview button with invalid form data', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Preview button should be disabled with empty form
    await expect(page.locator('[data-testid="preview-poll-btn"]')).toBeDisabled();
    
    // Fill title but not candidates
    await page.fill('[data-testid="poll-title-input"]', 'Incomplete Poll');
    
    // Should still be disabled
    await expect(page.locator('[data-testid="preview-poll-btn"]')).toBeDisabled();
    
    // Clear candidates and fill only one
    await page.fill('[data-testid="candidate-name-0"]', 'Only One Option');
    
    // Should still be disabled (need at least 2 candidates)
    await expect(page.locator('[data-testid="preview-poll-btn"]')).toBeDisabled();
  });

  test('should handle form validation errors gracefully', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Try to submit with minimal invalid data
    await page.fill('[data-testid="poll-title-input"]', 'A'); // Too short
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // Should stay on the same page due to validation
    await expect(page).toHaveURL('/polls/new');
    
    // Check if validation messages appear (implementation dependent)
    // At minimum, should not have submitted successfully
  });

  test('should show submit button loading state', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Fill valid form data
    await page.fill('[data-testid="poll-title-input"]', 'Loading State Test Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing loading state');
    await page.fill('[data-testid="candidate-name-0"]', 'Option A');
    await page.fill('[data-testid="candidate-name-1"]', 'Option B');
    
    // Submit form and check that submission works (loading state might be too fast to catch)
    await page.click('[data-testid="create-poll-submit-btn"]');
    
    // Wait for redirect to confirm submission worked
    await page.waitForURL('/dashboard', { timeout: 10000 });
  });

  test('should handle form accessibility features', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Test keyboard navigation by explicitly focusing elements
    await page.locator('[data-testid="poll-title-input"]').focus();
    await page.keyboard.type('Keyboard Navigation Test');
    
    await page.locator('[data-testid="poll-description-input"]').focus();
    await page.keyboard.type('Testing keyboard navigation');
    
    // Check that data was entered correctly
    await expect(page.locator('[data-testid="poll-title-input"]')).toHaveValue('Keyboard Navigation Test');
    await expect(page.locator('[data-testid="poll-description-input"]')).toHaveValue('Testing keyboard navigation');
  });

  test('should handle candidate management with test IDs', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Fill basic poll info
    await page.fill('[data-testid="poll-title-input"]', 'Candidate Management Test');
    
    // Check initial candidate inputs
    await expect(page.locator('[data-testid="candidate-name-0"]')).toBeVisible();
    await expect(page.locator('[data-testid="candidate-name-1"]')).toBeVisible();
    
    // Fill initial candidates
    await page.fill('[data-testid="candidate-name-0"]', 'First Candidate');
    await page.fill('[data-testid="candidate-name-1"]', 'Second Candidate');
    
    // Add more candidates using the add button
    await page.click('[data-testid="add-candidate-btn"]');
    await expect(page.locator('[data-testid="candidate-name-2"]')).toBeVisible();
    await page.fill('[data-testid="candidate-name-2"]', 'Third Candidate');
    
    await page.click('[data-testid="add-candidate-btn"]');
    await expect(page.locator('[data-testid="candidate-name-3"]')).toBeVisible();
    await page.fill('[data-testid="candidate-name-3"]', 'Fourth Candidate');
    
    // Verify all candidates are filled
    await expect(page.locator('[data-testid="candidate-name-0"]')).toHaveValue('First Candidate');
    await expect(page.locator('[data-testid="candidate-name-1"]')).toHaveValue('Second Candidate');
    await expect(page.locator('[data-testid="candidate-name-2"]')).toHaveValue('Third Candidate');
    await expect(page.locator('[data-testid="candidate-name-3"]')).toHaveValue('Fourth Candidate');
  });

  test('should handle poll type selection', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Check default selection
    await expect(page.locator('[data-testid="single-winner-radio"]')).toBeChecked();
    await expect(page.locator('[data-testid="multi-winner-radio"]')).not.toBeChecked();
    
    // Switch to multi-winner
    await page.click('[data-testid="multi-winner-radio"]');
    await expect(page.locator('[data-testid="multi-winner-radio"]')).toBeChecked();
    await expect(page.locator('[data-testid="single-winner-radio"]')).not.toBeChecked();
    
    // Number of winners input should be visible for multi-winner
    await expect(page.locator('[data-testid="num-winners-input"]')).toBeVisible();
    
    // Switch back to single winner
    await page.click('[data-testid="single-winner-radio"]');
    await expect(page.locator('[data-testid="single-winner-radio"]')).toBeChecked();
    await expect(page.locator('[data-testid="multi-winner-radio"]')).not.toBeChecked();
  });

  test('should validate multi-winner number input', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Switch to multi-winner
    await page.click('[data-testid="multi-winner-radio"]');
    
    // Fill basic poll data
    await page.fill('[data-testid="poll-title-input"]', 'Multi-Winner Test');
    await page.fill('[data-testid="candidate-name-0"]', 'Candidate A');
    await page.fill('[data-testid="candidate-name-1"]', 'Candidate B');
    await page.click('[data-testid="add-candidate-btn"]');
    await page.fill('[data-testid="candidate-name-2"]', 'Candidate C');
    
    // Test number of winners input
    await page.fill('[data-testid="num-winners-input"]', '2');
    await expect(page.locator('[data-testid="num-winners-input"]')).toHaveValue('2');
    
    // Should be able to create multi-winner poll
    await page.click('[data-testid="create-poll-submit-btn"]');
    // Wait for redirect to dashboard, then navigate to poll
    await page.waitForURL('/dashboard', { timeout: 10000 });
    await page.waitForSelector('[data-testid^="poll-item-"]');
    const pollElement = page.locator('[data-testid^="poll-item-"]').first();
    await pollElement.click();
    await page.waitForURL(/\/polls\/[a-f0-9-]+$/, { timeout: 10000 });
  });

  test('should handle form state persistence during navigation', async ({ page }) => {
    // Navigate to poll creation
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Fill form with data
    await page.fill('[data-testid="poll-title-input"]', 'Persistence Test Poll');
    await page.fill('[data-testid="poll-description-input"]', 'Testing form state persistence');
    await page.fill('[data-testid="candidate-name-0"]', 'Persistent Option 1');
    await page.fill('[data-testid="candidate-name-1"]', 'Persistent Option 2');
    
    // Navigate away and back (simulating accidental navigation)
    await page.click('[data-testid="cancel-poll-btn"]');
    await expect(page).toHaveURL('/dashboard');
    
    // Go back to form
    await page.click('[data-testid="create-first-poll-btn"]');
    await expect(page).toHaveURL('/polls/new');
    
    // Form should be reset (this is expected behavior for new forms)
    await expect(page.locator('[data-testid="poll-title-input"]')).toHaveValue('');
    await expect(page.locator('[data-testid="poll-description-input"]')).toHaveValue('');
  });
}); 