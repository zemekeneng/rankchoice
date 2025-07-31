import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  const testUser = {
    email: 'e2e-test@example.com',
    password: 'Test123!',
    name: 'E2E Test User'
  };

  test.beforeEach(async ({ page }) => {
    // Start with a clean slate - go to home page
    await page.goto('/');
  });

  test('should show login and register links when not authenticated', async ({ page }) => {
    await expect(page.locator('a[href="/login"]')).toBeVisible();
    await expect(page.locator('a[href="/register"]')).toBeVisible();
    await expect(page.locator('text=Welcome,')).not.toBeVisible();
  });

  test('should register a new user successfully', async ({ page }) => {
    // Go to register page
    await page.click('a[href="/register"]');
    await expect(page).toHaveURL('/register');

    // Fill registration form
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[type="password"]', testUser.password);

    // Submit form
    await page.click('button[type="submit"]');

    // Should redirect to dashboard after successful registration
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator(`text=Welcome, ${testUser.name}`)).toBeVisible();
  });

  test('should login with existing user credentials', async ({ page }) => {
    // First register a user (we'll reuse from previous test if it exists)
    await page.goto('/register');
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[name="name"]', testUser.name);
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');

    // Logout first
    await page.click('button:has-text("Logout")');
    await expect(page).toHaveURL('/');

    // Now test login
    await page.click('a[href="/login"]');
    await expect(page).toHaveURL('/login');

    // Fill login form
    await page.fill('input[type="email"]', testUser.email);
    await page.fill('input[type="password"]', testUser.password);

    // Submit form
    await page.click('button[type="submit"]');

    // Should redirect to dashboard after successful login
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator(`text=Welcome, ${testUser.name}`)).toBeVisible();
  });

  test('should show error for invalid login credentials', async ({ page }) => {
    await page.goto('/login');

    // Fill with invalid credentials
    await page.fill('input[type="email"]', 'invalid@example.com');
    await page.fill('input[type="password"]', 'wrongpassword');

    // Submit form
    await page.click('button[type="submit"]');

    // Should show error message and stay on login page
    await expect(page).toHaveURL('/login');
    await expect(page.locator('text=Login failed')).toBeVisible();
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
    // First login
    await page.goto('/register');
    await page.fill('input[type="email"]', `logout-${Date.now()}@example.com`);
    await page.fill('input[name="name"]', 'Logout Test User');
    await page.fill('input[type="password"]', testUser.password);
    await page.click('button[type="submit"]');

    // Verify logged in
    await expect(page).toHaveURL('/dashboard');
    
    // Logout
    await page.click('button:has-text("Logout")');

    // Should redirect to home page and show login/register links
    await expect(page).toHaveURL('/');
    await expect(page.locator('a[href="/login"]')).toBeVisible();
    await expect(page.locator('a[href="/register"]')).toBeVisible();
  });
}); 