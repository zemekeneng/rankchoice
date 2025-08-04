import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  // Function to generate unique test data for each test to avoid collisions
  function generateTestUser(testPrefix: string = 'auth') {
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(7);
    const extraRandom = Math.random().toString(36).substring(2, 8); // Additional randomness
    const microTime = performance.now().toString().replace('.', '').slice(-6); // High precision timing
    return {
      email: `${testPrefix}-${timestamp}-${randomId}-${extraRandom}-${microTime}@example.com`,
      password: 'Test123!',
      name: `${testPrefix.charAt(0).toUpperCase()}${testPrefix.slice(1)} Test User`
    };
  }

  test.beforeEach(async ({ page }) => {
    // Start with a clean slate - go to home page
    await page.goto('/');
  });

  test('should show login and register links when not authenticated', async ({ page }) => {
    await expect(page.locator('[data-testid="login-link"]')).toBeVisible();
    await expect(page.locator('[data-testid="register-link"]')).toBeVisible();
    await expect(page.locator('[data-testid="welcome-text"]')).not.toBeVisible();
  });

  test('should register a new user successfully', async ({ page }) => {
    const testUser = generateTestUser('register');
    console.log(`🔍 [REGISTER TEST] Starting with email: ${testUser.email}`);
    
    // Enable console logging from the page
    page.on('console', msg => console.log(`📄 [PAGE]: ${msg.text()}`));
    
    // Go to register page
    console.log('🔍 [REGISTER TEST] Clicking register link');
    await page.click('[data-testid="register-link"]');
    await expect(page).toHaveURL('/register');
    console.log('🔍 [REGISTER TEST] On register page');

    // Fill registration form
    console.log('🔍 [REGISTER TEST] Filling form fields');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    
    console.log('🔍 [REGISTER TEST] Submitting form');
    await page.click('[data-testid="register-submit-btn"]');

    // Check for errors first
    const errorElement = page.locator('[data-testid="register-error"]');
    const hasError = await errorElement.isVisible().catch(() => false);
    if (hasError) {
      const errorText = await errorElement.textContent();
      console.log(`❌ [REGISTER TEST] Found error: ${errorText}`);
    }

    console.log('🔍 [REGISTER TEST] Waiting for dashboard redirect');
    // Should redirect to dashboard after successful registration
    await expect(page).toHaveURL('/dashboard');
    console.log('🔍 [REGISTER TEST] Successfully redirected to dashboard');
    await expect(page.locator('[data-testid="welcome-text"]')).toContainText(`Welcome, ${testUser.name}`);
    console.log('🔍 [REGISTER TEST] Test completed successfully');
  });

  test('should login with existing user credentials', async ({ page }) => {
    // Create a unique user for this test
    const loginTestUser = generateTestUser('login');

    // First register a user
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', loginTestUser.email);
    await page.fill('[data-testid="name-input"]', loginTestUser.name);
    await page.fill('[data-testid="register-password-input"]', loginTestUser.password);
    await page.fill('[data-testid="confirm-password-input"]', loginTestUser.password);
    await page.click('[data-testid="register-submit-btn"]');

    // Should be logged in and on dashboard
    await expect(page).toHaveURL('/dashboard');

    // Logout first
    await page.click('[data-testid="logout-btn"]');
    
    // Wait for logout to complete - app might redirect to login instead of home
    await page.waitForURL(/\/(login)?$/);
    
    // Navigate to login if not already there
    if (page.url().endsWith('/')) {
      await page.click('[data-testid="login-link"]');
    }
    await expect(page).toHaveURL('/login');

    // Fill login form
    await page.fill('[data-testid="email-input"]', loginTestUser.email);
    await page.fill('[data-testid="password-input"]', loginTestUser.password);

    // Submit form
    await page.click('[data-testid="login-submit-btn"]');

    // Should redirect to dashboard after successful login
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('[data-testid="welcome-text"]')).toContainText(`Welcome, ${loginTestUser.name}`);
  });

  test('should show error for invalid login credentials', async ({ page }) => {
    await page.goto('/login');

    // Fill with invalid credentials
    await page.fill('[data-testid="email-input"]', 'invalid@example.com');
    await page.fill('[data-testid="password-input"]', 'wrongpassword');

    // Submit form
    await page.click('[data-testid="login-submit-btn"]');

    // Should show error message and stay on login page
    await expect(page).toHaveURL('/login');
    await expect(page.locator('[data-testid="login-error"]')).toBeVisible();
  });

  test('should protect authenticated routes', async ({ page }) => {
    // Try to access dashboard without authentication
    await page.goto('/dashboard');
    
    // Should redirect to login
    await expect(page).toHaveURL('/login');
    
    // Try to access poll creation without authentication
    await page.goto('/polls/new');
    
    // Should redirect to login
    await expect(page).toHaveURL('/login');
  });

  test('should logout successfully', async ({ page }) => {
    // Create a unique user for this test
    const logoutTestUser = generateTestUser('logout');

    // First register and login
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', logoutTestUser.email);
    await page.fill('[data-testid="name-input"]', logoutTestUser.name);
    await page.fill('[data-testid="register-password-input"]', logoutTestUser.password);
    await page.fill('[data-testid="confirm-password-input"]', logoutTestUser.password);
    await page.click('[data-testid="register-submit-btn"]');

    // Verify logged in (wait longer for concurrency issue)
    await expect(page).toHaveURL('/dashboard', { timeout: 15000 });
    
    // Logout
    await page.click('[data-testid="logout-btn"]');

    // Should be logged out (either on home or login page)
    await page.waitForURL(/\/(login)?$/, { timeout: 1000 });
    
    // Navigate to home page to verify logout state
    await page.goto('/');
    await expect(page.locator('[data-testid="login-link"]')).toBeVisible();
    await expect(page.locator('[data-testid="register-link"]')).toBeVisible();
    await expect(page.locator('[data-testid="welcome-text"]')).not.toBeVisible();
  });
}); 