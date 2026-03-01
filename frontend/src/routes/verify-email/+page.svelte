<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiClient } from '$lib/api/client.js';
	import { authStore } from '$lib/stores/auth.svelte.js';

	let status = $state<'loading' | 'success' | 'error'>('loading');
	let errorMessage = $state('');

	$effect(() => {
		const token = $page.url.searchParams.get('token');
		if (!token) {
			status = 'error';
			errorMessage = 'No verification token provided.';
			return;
		}

		verifyEmail(token);
	});

	async function verifyEmail(token: string) {
		try {
			await apiClient.verifyEmail(token);
			authStore.setEmailVerified();
			status = 'success';
		} catch (error: any) {
			status = 'error';
			errorMessage = error?.message || 'Failed to verify email. The link may be expired or invalid.';
		}
	}
</script>

<svelte:head>
	<title>Verify Email - RankChoice</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8 text-center">
		{#if status === 'loading'}
			<div>
				<svg class="animate-spin mx-auto h-12 w-12 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
				<h2 class="mt-6 text-2xl font-bold text-gray-900">Verifying your email...</h2>
				<p class="mt-2 text-gray-600">Please wait a moment.</p>
			</div>
		{:else if status === 'success'}
			<div>
				<div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100">
					<svg class="h-8 w-8 text-green-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
					</svg>
				</div>
				<h2 class="mt-6 text-2xl font-bold text-gray-900">Email verified!</h2>
				<p class="mt-2 text-gray-600">Your email address has been successfully verified.</p>
				<div class="mt-6">
					<a
						href="/dashboard"
						class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
					>
						Go to Dashboard
					</a>
				</div>
			</div>
		{:else}
			<div>
				<div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-red-100">
					<svg class="h-8 w-8 text-red-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</div>
				<h2 class="mt-6 text-2xl font-bold text-gray-900">Verification Failed</h2>
				<p class="mt-2 text-gray-600">{errorMessage}</p>
				<div class="mt-6 space-x-4">
					<a
						href="/login"
						class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
					>
						Go to Login
					</a>
				</div>
			</div>
		{/if}
	</div>
</div>
