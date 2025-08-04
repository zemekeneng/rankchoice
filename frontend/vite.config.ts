import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	
	// Server optimizations for better concurrency
	server: {
		// Increase connection limits and timeouts
		host: '0.0.0.0', // Allow external connections
		port: 5173,
		strictPort: true,
		
		// Performance optimizations
		hmr: {
			// Reduce HMR overhead during testing
			overlay: false,
		},
		
		// Connection settings for better concurrency
		headers: {
			'Connection': 'keep-alive',
			'Keep-Alive': 'timeout=30, max=1000'
		},
		
		// Warm up frequently used files for better performance
		warmup: {
			clientFiles: [
				'./src/app.html',
				'./src/routes/+layout.svelte',
				'./src/routes/+page.svelte',
				'./src/routes/register/+page.svelte',
				'./src/routes/login/+page.svelte',
				'./src/routes/dashboard/+page.svelte',
				'./src/lib/stores/auth.svelte.ts',
				'./src/lib/api/client.ts'
			]
		},
		
		// Don't open browser during testing
		open: false
	},
	
	// Resolve optimizations to reduce filesystem checks
	resolve: {
		// Reduce extension checking - be explicit in imports
		extensions: ['.js', '.ts', '.svelte', '.json'],
		alias: {
			// Avoid barrel file imports for better performance
			'$lib': './src/lib',
			'$components': './src/lib/components'
		}
	},
	
	// Build optimizations that also help dev server
	optimizeDeps: {
		// Pre-bundle dependencies for faster loading
		include: [
			'@sveltejs/kit',
			'svelte',
			'svelte/store',
			'tailwindcss'
		],
		// Reduce dependency scanning overhead
		force: false,
		// Exclude problematic dependencies that might slow down dev server
		exclude: []
	},
	
	// Development-specific optimizations
	define: {
		// Reduce bundle analysis during development
		'process.env.NODE_ENV': '"development"'
	},
	
	// Caching optimizations
	cacheDir: 'node_modules/.vite',
	
	// Logging configuration for performance debugging
	logLevel: 'info',
	clearScreen: false,
	
	// Additional performance optimizations
	esbuild: {
		// Faster JavaScript transformation
		target: 'esnext',
		legalComments: 'none'
	},
	
	// Build configuration (helps with overall performance)
	build: {
		// Reduce build overhead that can affect dev server
		target: 'esnext',
		minify: false, // Faster builds during dev
		sourcemap: true,
		rollupOptions: {
			// Optimize chunk splitting for better caching
			output: {
				manualChunks: {
					vendor: ['svelte', '@sveltejs/kit'],
					utils: ['tailwindcss']
				}
			}
		}
	}
	// Vitest configuration disabled to avoid conflicts with Playwright E2E tests
	// test: {
	// 	expect: { requireAssertions: true },
	// 	projects: [
	// 		{
	// 			extends: './vite.config.ts',
	// 			test: {
	// 				name: 'client',
	// 				environment: 'browser',
	// 				browser: {
	// 					enabled: true,
	// 					provider: 'playwright',
	// 					instances: [{ browser: 'chromium' }]
	// 				},
	// 				include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
	// 				exclude: ['src/lib/server/**', 'e2e/**'],
	// 				setupFiles: ['./vitest-setup-client.ts']
	// 			}
	// 		},
	// 		{
	// 			extends: './vite.config.ts',
	// 			test: {
	// 				name: 'server',
	// 				environment: 'node',
	// 				include: ['src/**/*.{test,spec}.{js,ts}'],
	// 				exclude: ['src/**/*.svelte.{test,spec}.{js,ts}', 'e2e/**']
	// 			}
	// 		}
	// 	]
	// }
});
