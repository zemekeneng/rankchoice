import { test, expect } from '@playwright/test';

test.describe('Homepage and Landing Page', () => {
  test('should display hero section with correct content', async ({ page }) => {
    await page.goto('/');
    
    // Verify hero section is visible
    await expect(page.locator('[data-testid="hero-section"]')).toBeVisible();
    
    // Check hero title
    await expect(page.locator('[data-testid="hero-title"]')).toBeVisible();
    await expect(page.locator('[data-testid="hero-title"]')).toContainText('RankChoice');
    
    // Check hero description
    await expect(page.locator('[data-testid="hero-description"]')).toBeVisible();
    await expect(page.locator('[data-testid="hero-description"]')).toContainText('ranked-choice voting');
    
    // Verify hero actions section
    await expect(page.locator('[data-testid="hero-actions"]')).toBeVisible();
  });

  test('should show appropriate hero buttons for unauthenticated users', async ({ page }) => {
    await page.goto('/');
    
    // Should show Get Started and Sign In buttons for unauthenticated users
    await expect(page.locator('[data-testid="hero-get-started-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="hero-get-started-btn"]')).toContainText('Get Started');
    
    await expect(page.locator('[data-testid="hero-sign-in-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="hero-sign-in-btn"]')).toContainText('Sign In');
    
    // Should not show dashboard button
    await expect(page.locator('[data-testid="hero-dashboard-btn"]')).not.toBeVisible();
  });

  test('should navigate to registration when clicking Get Started', async ({ page }) => {
    // Clear any existing authentication state
    await page.goto('/');
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
    await page.reload();
    
    // Ensure we see the Get Started button (unauthenticated state)
    await expect(page.locator('[data-testid="hero-get-started-btn"]')).toBeVisible();
    
    // Click and navigate
    await page.click('[data-testid="hero-get-started-btn"]');
    await page.waitForURL('/register');
    
    await expect(page.locator('[data-testid="register-heading"]')).toBeVisible();
  });

  test('should navigate to login when clicking Sign In', async ({ page }) => {
    // Clear any existing authentication state
    await page.goto('/');
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
    await page.reload();
    
    // Ensure we see the Sign In button (unauthenticated state)
    await expect(page.locator('[data-testid="hero-sign-in-btn"]')).toBeVisible();
    
    // Click and navigate
    await page.click('[data-testid="hero-sign-in-btn"]');
    await page.waitForURL('/login');
    
    await expect(page.locator('[data-testid="login-heading"]')).toBeVisible();
  });

  test('should display features section with all features', async ({ page }) => {
    await page.goto('/');
    
    // Verify features section
    await expect(page.locator('[data-testid="features-section"]')).toBeVisible();
    await expect(page.locator('[data-testid="features-heading"]')).toContainText('Why Choose RankChoice?');
    await expect(page.locator('[data-testid="features-subheading"]')).toContainText('transparency, security, and ease of use');
    
    // Check all feature cards are present
    await expect(page.locator('[data-testid="features-grid"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-poll-creation"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-secure-voting"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-real-time-results"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-drag-drop"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-email-distribution"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-verified-results"]')).toBeVisible();
    
    // Verify feature content
    await expect(page.locator('[data-testid="feature-poll-creation"]')).toContainText('Easy Poll Creation');
    await expect(page.locator('[data-testid="feature-secure-voting"]')).toContainText('Secure Voting');
    await expect(page.locator('[data-testid="feature-real-time-results"]')).toContainText('Real-time Results');
    await expect(page.locator('[data-testid="feature-drag-drop"]')).toContainText('Drag & Drop Ranking');
    await expect(page.locator('[data-testid="feature-email-distribution"]')).toContainText('Email Distribution');
    await expect(page.locator('[data-testid="feature-verified-results"]')).toContainText('Verified Results');
  });

  test('should display how it works section', async ({ page }) => {
    await page.goto('/');
    
    // Verify how it works section
    await expect(page.locator('[data-testid="how-it-works-section"]')).toBeVisible();
    await expect(page.locator('[data-testid="how-it-works-heading"]')).toContainText('How Ranked-Choice Voting Works');
    await expect(page.locator('[data-testid="how-it-works-subheading"]')).toContainText('Simple, fair, and transparent');
    
    // Check all steps are present
    await expect(page.locator('[data-testid="how-it-works-steps"]')).toBeVisible();
    await expect(page.locator('[data-testid="step-rank-candidates"]')).toBeVisible();
    await expect(page.locator('[data-testid="step-count-first-choices"]')).toBeVisible();
    await expect(page.locator('[data-testid="step-redistribute-votes"]')).toBeVisible();
    
    // Verify step content
    await expect(page.locator('[data-testid="step-rank-candidates"]')).toContainText('Voters Rank Candidates');
    await expect(page.locator('[data-testid="step-count-first-choices"]')).toContainText('Count First Choices');
    await expect(page.locator('[data-testid="step-redistribute-votes"]')).toContainText('Redistribute Votes');
  });

  test('should display call-to-action section for unauthenticated users', async ({ page }) => {
    await page.goto('/');
    
    // Should show CTA section for unauthenticated users
    await expect(page.locator('[data-testid="cta-section"]')).toBeVisible();
    await expect(page.locator('[data-testid="cta-heading"]')).toContainText('Ready to get started?');
    await expect(page.locator('[data-testid="cta-description"]')).toContainText('fair, transparent elections');
    
    // Verify CTA button
    await expect(page.locator('[data-testid="cta-signup-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="cta-signup-btn"]')).toContainText('Sign up for free');
  });

  test('should navigate to registration from CTA button', async ({ page }) => {
    // Clear any existing authentication state
    await page.goto('/');
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
    await page.reload();
    
    // Ensure we see the CTA signup button (unauthenticated state)
    await expect(page.locator('[data-testid="cta-signup-btn"]')).toBeVisible();
    
    // Click and navigate
    await page.click('[data-testid="cta-signup-btn"]');
    await page.waitForURL('/register');
    
    await expect(page.locator('[data-testid="register-heading"]')).toBeVisible();
  });

  test('should show dashboard button for authenticated users', async ({ page }) => {
    // Create and login a test user first
    const timestamp = Date.now();
    const testUser = {
      email: `homepage-auth-${timestamp}@example.com`,
      password: 'Test123!',
      name: 'Homepage Auth Test User'
    };

    // Register
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
    
    // Now visit homepage as authenticated user
    await page.goto('/');
    
    // Should show dashboard button instead of Get Started/Sign In
    await expect(page.locator('[data-testid="hero-dashboard-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="hero-dashboard-btn"]')).toContainText('Go to Dashboard');
    
    // Should not show Get Started/Sign In buttons
    await expect(page.locator('[data-testid="hero-get-started-btn"]')).not.toBeVisible();
    await expect(page.locator('[data-testid="hero-sign-in-btn"]')).not.toBeVisible();
    
    // Should not show CTA section for authenticated users
    await expect(page.locator('[data-testid="cta-section"]')).not.toBeVisible();
  });

  test('should navigate to dashboard when authenticated user clicks dashboard button', async ({ page }) => {
    // Create and login a test user first
    const timestamp = Date.now();
    const testUser = {
      email: `homepage-dashboard-${timestamp}@example.com`,
      password: 'Test123!',
      name: 'Homepage Dashboard Test User'
    };

    // Register
    await page.goto('/register');
    await page.fill('[data-testid="register-email-input"]', testUser.email);
    await page.fill('[data-testid="name-input"]', testUser.name);
    await page.fill('[data-testid="register-password-input"]', testUser.password);
    await page.fill('[data-testid="confirm-password-input"]', testUser.password);
    await page.click('[data-testid="register-submit-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
    
    // Visit homepage and click dashboard button
    await page.goto('/');
    await page.click('[data-testid="hero-dashboard-btn"]');
    
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('[data-testid="welcome-text"]')).toContainText(`Welcome, ${testUser.name}`);
  });

  test('should have responsive design on mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');
    
    // All main sections should still be visible on mobile
    await expect(page.locator('[data-testid="hero-section"]')).toBeVisible();
    await expect(page.locator('[data-testid="features-section"]')).toBeVisible();
    await expect(page.locator('[data-testid="how-it-works-section"]')).toBeVisible();
    
    // Hero buttons should be visible and properly sized
    await expect(page.locator('[data-testid="hero-get-started-btn"]')).toBeVisible();
    await expect(page.locator('[data-testid="hero-sign-in-btn"]')).toBeVisible();
    
    // Features should still be readable
    await expect(page.locator('[data-testid="feature-poll-creation"]')).toBeVisible();
    await expect(page.locator('[data-testid="feature-secure-voting"]')).toBeVisible();
  });

  test('should navigate properly with header links', async ({ page }) => {
    await page.goto('/');
    
    // Test home link
    await expect(page.locator('[data-testid="home-link"]')).toBeVisible();
    await page.click('[data-testid="home-link"]');
    await expect(page).toHaveURL('/');
    
    // Test login link from header
    await expect(page.locator('[data-testid="login-link"]')).toBeVisible();
    await page.click('[data-testid="login-link"]');
    await expect(page).toHaveURL('/login');
    
    // Go back and test register link
    await page.goto('/');
    await expect(page.locator('[data-testid="register-link"]')).toBeVisible();
    await page.click('[data-testid="register-link"]');
    await expect(page).toHaveURL('/register');
  });
}); 