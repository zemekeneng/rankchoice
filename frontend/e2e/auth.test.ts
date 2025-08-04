import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  // Function to generate unique test data for each test to avoid collisions
  function generateTestUser(testPrefix: string = 'auth') {
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substring(7);
    return {
      email: `${testPrefix}-${timestamp}-${randomId}@example.com`,
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
    
    // Go to register page
    await page.click('[data-testid="register-link"]');
    await expect(page).toHaveURL('/register');

    // Fill registration form
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);

    // Submit form
    await page.click('[data-testid="register-submit-btn"]');

    // Should redirect to dashboard after successful registration
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('[data-testid="welcome-text"]')).toContainText(`Welcome, ${testUser.name}`);
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

    // Verify logged in
    await expect(page).toHaveURL('/dashboard');
    
    // Logout
    await page.click('[data-testid="logout-btn"]');

    // Should be logged out (either on home or login page)
    await page.waitForURL(/\/(login)?$/);
    
    // Navigate to home page to verify logout state
    await page.goto('/');
    await expect(page.locator('[data-testid="login-link"]')).toBeVisible();
    await expect(page.locator('[data-testid="register-link"]')).toBeVisible();
    await expect(page.locator('[data-testid="welcome-text"]')).not.toBeVisible();
  });
}); 