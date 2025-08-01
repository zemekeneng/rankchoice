import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: 'e2e',
	fullyParallel: true,
	retries: 0,
	workers: process.env.CI ? 2 : undefined, // Use default workers (CPU cores) locally, 2 in CI
	reporter: [['list'], ['html', { open: 'never' }]],
	use: {
		baseURL: 'http://localhost:5173',
		trace: 'on-first-retry',
		actionTimeout: 10000,
		navigationTimeout: 30000,
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] },
		},
	],
	webServer: {
		command: 'npm run dev',
		port: 5173,
		reuseExistingServer: true,
	},
});
