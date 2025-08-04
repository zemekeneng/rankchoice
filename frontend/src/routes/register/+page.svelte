<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte.js';
	import type { RegisterForm, FormErrors } from '$lib/types.js';
	import { APIError } from '$lib/api/client.js';

	// Form state using Svelte 5 $state
	let form = $state<RegisterForm>({
		email: '',
		password: '',
		confirmPassword: '',
		name: ''
	});

	let errors = $state<FormErrors>({});
	let isSubmitting = $state(false);

	// Validation functions
	function validateEmail(email: string): string[] {
		const emailErrors: string[] = [];
		
		if (!email.trim()) {
			emailErrors.push('Email is required');
		} else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
			emailErrors.push('Please enter a valid email address');
		}
		
		return emailErrors;
	}

	function validatePassword(password: string): string[] {
		const passwordErrors: string[] = [];
		
		if (!password) {
			passwordErrors.push('Password is required');
		} else if (password.length < 6) {
			passwordErrors.push('Password must be at least 6 characters');
		} else if (!/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/.test(password)) {
			passwordErrors.push('Password must contain at least one uppercase letter, one lowercase letter, and one number');
		}
		
		return passwordErrors;
	}

	function validateConfirmPassword(password: string, confirmPassword: string): string[] {
		const confirmErrors: string[] = [];
		
		if (!confirmPassword) {
			confirmErrors.push('Please confirm your password');
		} else if (password !== confirmPassword) {
			confirmErrors.push('Passwords do not match');
		}
		
		return confirmErrors;
	}

	function validateName(name: string): string[] {
		const nameErrors: string[] = [];
		
		if (!name.trim()) {
			nameErrors.push('Name is required');
		} else if (name.trim().length < 2) {
			nameErrors.push('Name must be at least 2 characters');
		}
		
		return nameErrors;
	}

	function validateForm(): boolean {
		errors = {
			email: validateEmail(form.email),
			password: validatePassword(form.password),
			confirmPassword: validateConfirmPassword(form.password, form.confirmPassword),
			name: validateName(form.name)
		};

		return Object.values(errors).every(fieldErrors => fieldErrors.length === 0);
	}

	// Handle form submission with robust timing protection
	async function handleSubmit(event: Event) {
		// Immediate and aggressive prevention of default form submission
		try {
			if (event) {
				event.preventDefault();
				event.stopPropagation();
				event.stopImmediatePropagation();
			}
		} catch (e) {
			console.warn('Event prevention error:', e);
		}
		
		// Double-check we're not already submitting to prevent double submission
		if (isSubmitting) {
			return false;
		}
		
		if (!validateForm()) {
			return false;
		}

		isSubmitting = true;
		authStore.clearError();

		try {
			await authStore.register({
				email: form.email.trim(),
				password: form.password,
				name: form.name.trim()
			});
			
			// Navigation is handled by the auth store
		} catch (error) {
			console.error('Registration error:', error);
			// Error is handled by the auth store - just continue
			// The error is displayed via authStore.error in the template
		} finally {
			isSubmitting = false;
		}
		
		return false; // Ensure form doesn't submit
	}

	// Additional backup submit handler for submit button clicks
	async function handleSubmitButton(event: Event) {
		event.preventDefault();
		event.stopPropagation();
		await handleSubmit(event);
		return false;
	}

	// Clear field error when user starts typing
	function clearFieldError(field: keyof RegisterForm) {
		if (errors[field]) {
			errors[field] = [];
		}
	}

	// Redirect if already authenticated
	$effect(() => {
		if (authStore.isAuthenticated) {
			goto('/dashboard');
		}
	});
</script>

<svelte:head>
	<title>Register - RankChoice</title>
	<meta name="description" content="Create your RankChoice account" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div>
			<h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900" data-testid="register-heading">
				Create your account
			</h2>
			<p class="mt-2 text-center text-sm text-gray-600">
				Or
				<a href="/login" class="font-medium text-indigo-600 hover:text-indigo-500" data-testid="sign-in-link">
					sign in to your existing account
				</a>
			</p>
		</div>

		<form class="mt-8 space-y-6" onsubmit={handleSubmit}>
			<div class="space-y-4">
				<div>
					<label for="name" class="block text-sm font-medium text-gray-700">
						Full Name
					</label>
					<input
						id="name"
						name="name"
						type="text"
						data-testid="name-input"
						autocomplete="name"
						required
						class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
						class:border-red-300={errors.name?.length > 0}
						class:focus:ring-red-500={errors.name?.length > 0}
						class:focus:border-red-500={errors.name?.length > 0}
						placeholder="Enter your full name"
						bind:value={form.name}
						oninput={() => clearFieldError('name')}
						disabled={isSubmitting}
					/>
					{#if errors.name?.length > 0}
						<div class="mt-1 text-red-600 text-sm">
							{#each errors.name as error}
								<p>{error}</p>
							{/each}
						</div>
					{/if}
				</div>

				<div>
					<label for="email" class="block text-sm font-medium text-gray-700">
						Email Address
					</label>
					<input
						id="email"
						name="email"
						type="email"
						data-testid="register-email-input"
						autocomplete="email"
						required
						class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
						class:border-red-300={errors.email?.length > 0}
						class:focus:ring-red-500={errors.email?.length > 0}
						class:focus:border-red-500={errors.email?.length > 0}
						placeholder="Enter your email address"
						bind:value={form.email}
						oninput={() => clearFieldError('email')}
						disabled={isSubmitting}
					/>
					{#if errors.email?.length > 0}
						<div class="mt-1 text-red-600 text-sm">
							{#each errors.email as error}
								<p>{error}</p>
							{/each}
						</div>
					{/if}
				</div>

				<div>
					<label for="password" class="block text-sm font-medium text-gray-700">
						Password
					</label>
					<input
						id="password"
						name="password"
						type="password"
						data-testid="register-password-input"
						autocomplete="new-password"
						required
						class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
						class:border-red-300={errors.password?.length > 0}
						class:focus:ring-red-500={errors.password?.length > 0}
						class:focus:border-red-500={errors.password?.length > 0}
						placeholder="Enter your password"
						bind:value={form.password}
						oninput={() => clearFieldError('password')}
						disabled={isSubmitting}
					/>
					{#if errors.password?.length > 0}
						<div class="mt-1 text-red-600 text-sm">
							{#each errors.password as error}
								<p>{error}</p>
							{/each}
						</div>
					{/if}
				</div>

				<div>
					<label for="confirmPassword" class="block text-sm font-medium text-gray-700">
						Confirm Password
					</label>
					<input
						id="confirmPassword"
						name="confirmPassword"
						type="password"
						data-testid="confirm-password-input"
						autocomplete="new-password"
						required
						class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 focus:z-10 sm:text-sm"
						class:border-red-300={errors.confirmPassword?.length > 0}
						class:focus:ring-red-500={errors.confirmPassword?.length > 0}
						class:focus:border-red-500={errors.confirmPassword?.length > 0}
						placeholder="Confirm your password"
						bind:value={form.confirmPassword}
						oninput={() => clearFieldError('confirmPassword')}
						disabled={isSubmitting}
					/>
					{#if errors.confirmPassword?.length > 0}
						<div class="mt-1 text-red-600 text-sm">
							{#each errors.confirmPassword as error}
								<p>{error}</p>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<!-- Global Error -->
			{#if authStore.error}
				<div class="bg-red-50 border border-red-200 rounded-md p-4" data-testid="register-error">
					<div class="flex">
						<div class="flex-shrink-0">
							<svg class="h-5 w-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
								<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
							</svg>
						</div>
						<div class="ml-3">
							<h3 class="text-sm font-medium text-red-800">
								Registration Failed
							</h3>
							<div class="mt-2 text-sm text-red-700">
								<p>{authStore.error}</p>
							</div>
						</div>
					</div>
				</div>
			{/if}

			<div>
				<button
					type="button"
					data-testid="register-submit-btn"
					disabled={isSubmitting || authStore.isLoading}
					onclick={handleSubmitButton}
					class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					{#if isSubmitting || authStore.isLoading}
						<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						Creating account...
					{:else}
						Create account
					{/if}
				</button>
			</div>

			<div class="text-center">
				<a href="/" class="font-medium text-indigo-600 hover:text-indigo-500" data-testid="register-back-home-link">
					← Back to home
				</a>
			</div>
		</form>
	</div>
</div>

<style>
	/* Additional custom styles can go here */
</style> 