<script lang="ts">
	import { page } from '$app/stores';
	import { apiClient } from '$lib/api/client.js';

	let password = $state('');
	let confirmPassword = $state('');
	let isSubmitting = $state(false);
	let success = $state(false);
	let error = $state('');
	let fieldErrors = $state<{ password: string[]; confirmPassword: string[] }>({
		password: [],
		confirmPassword: []
	});

	let token = $derived($page.url.searchParams.get('token'));

	function validatePassword(pw: string): string[] {
		const errs: string[] = [];
		if (!pw) {
			errs.push('Password is required');
		} else if (pw.length < 6) {
			errs.push('Password must be at least 6 characters');
		} else if (!/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/.test(pw)) {
			errs.push('Password must contain at least one uppercase letter, one lowercase letter, and one number');
		}
		return errs;
	}

	function validateForm(): boolean {
		fieldErrors = {
			password: validatePassword(password),
			confirmPassword: !confirmPassword
				? ['Please confirm your password']
				: password !== confirmPassword
					? ['Passwords do not match']
					: []
		};
		return fieldErrors.password.length === 0 && fieldErrors.confirmPassword.length === 0;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (isSubmitting) return;

		error = '';
		if (!validateForm()) return;

		if (!token) {
			error = 'No reset token found. Please use the link from your email.';
			return;
		}

		isSubmitting = true;

		try {
			await apiClient.resetPassword(token, password);
			success = true;
		} catch (e: any) {
			error = e?.message || 'Failed to reset password. The link may be expired or invalid.';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>Reset Password - RankedChoice</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		{#if !token}
			<div class="text-center">
				<div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-red-100">
					<svg class="h-8 w-8 text-red-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</div>
				<h2 class="mt-6 text-2xl font-bold text-gray-900">Invalid Link</h2>
				<p class="mt-2 text-gray-600">This password reset link is invalid. Please request a new one.</p>
				<div class="mt-6">
					<a href="/forgot-password" class="font-medium text-indigo-600 hover:text-indigo-500">
						Request new reset link
					</a>
				</div>
			</div>
		{:else if success}
			<div class="text-center">
				<div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100">
					<svg class="h-8 w-8 text-green-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
					</svg>
				</div>
				<h2 class="mt-6 text-2xl font-bold text-gray-900">Password Reset!</h2>
				<p class="mt-2 text-gray-600">Your password has been successfully reset. You can now sign in with your new password.</p>
				<div class="mt-6">
					<a
						href="/login"
						class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
					>
						Go to Login
					</a>
				</div>
			</div>
		{:else}
			<div>
				<h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
					Set a new password
				</h2>
				<p class="mt-2 text-center text-sm text-gray-600">
					Enter your new password below.
				</p>
			</div>

			<form class="mt-8 space-y-6" onsubmit={handleSubmit}>
				<div class="space-y-4">
					<div>
						<label for="password" class="block text-sm font-medium text-gray-700">
							New Password
						</label>
						<input
							id="password"
							name="password"
							type="password"
							autocomplete="new-password"
							required
							class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
							class:border-red-300={fieldErrors.password.length > 0}
							placeholder="Enter your new password"
							bind:value={password}
							oninput={() => { fieldErrors.password = []; error = ''; }}
							disabled={isSubmitting}
						/>
						{#if fieldErrors.password.length > 0}
							<div class="mt-1 text-red-600 text-sm">
								{#each fieldErrors.password as err}
									<p>{err}</p>
								{/each}
							</div>
						{/if}
					</div>

					<div>
						<label for="confirmPassword" class="block text-sm font-medium text-gray-700">
							Confirm New Password
						</label>
						<input
							id="confirmPassword"
							name="confirmPassword"
							type="password"
							autocomplete="new-password"
							required
							class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
							class:border-red-300={fieldErrors.confirmPassword.length > 0}
							placeholder="Confirm your new password"
							bind:value={confirmPassword}
							oninput={() => { fieldErrors.confirmPassword = []; error = ''; }}
							disabled={isSubmitting}
						/>
						{#if fieldErrors.confirmPassword.length > 0}
							<div class="mt-1 text-red-600 text-sm">
								{#each fieldErrors.confirmPassword as err}
									<p>{err}</p>
								{/each}
							</div>
						{/if}
					</div>
				</div>

				{#if error}
					<div class="bg-red-50 border border-red-200 rounded-md p-4">
						<div class="flex">
							<div class="flex-shrink-0">
								<svg class="h-5 w-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
									<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
								</svg>
							</div>
							<div class="ml-3">
								<div class="text-sm text-red-700">
									<p>{error}</p>
								</div>
							</div>
						</div>
					</div>
				{/if}

				<div>
					<button
						type="submit"
						disabled={isSubmitting}
						class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{#if isSubmitting}
							<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							Resetting...
						{:else}
							Reset Password
						{/if}
					</button>
				</div>

				<div class="text-center">
					<a href="/login" class="font-medium text-indigo-600 hover:text-indigo-500">
						Back to login
					</a>
				</div>
			</form>
		{/if}
	</div>
</div>
