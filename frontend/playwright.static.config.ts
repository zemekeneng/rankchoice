import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: 'e2e',
	fullyParallel: true,
	retries: 0,
	workers: process.env.CI ? 4 : 4, // Higher concurrency for static server
	reporter: [['list'], ['html', { open: 'never' }]],
	use: {
		baseURL: 'http://localhost:5174',
		trace: 'on-first-retry',
		actionTimeout: 5000, // Shorter timeouts since static server is more reliable
		navigationTimeout: 15000,
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] },
		},
	],
	webServer: {
		command: 'npm run build:serve',
		port: 5174,
		reuseExistingServer: false, // Always restart server for clean state
		stdout: 'ignore', // Suppress server logs
		stderr: 'pipe', // Keep error logs for debugging
		timeout: 10000, // 10 second timeout for server startup
	},
});