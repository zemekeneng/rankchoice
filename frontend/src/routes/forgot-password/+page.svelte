<script lang="ts">
	import { apiClient } from '$lib/api/client.js';

	let email = $state('');
	let isSubmitting = $state(false);
	let submitted = $state(false);
	let error = $state('');

	function validateEmail(email: string): boolean {
		return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (isSubmitting) return;

		error = '';

		if (!email.trim()) {
			error = 'Email is required';
			return;
		}
		if (!validateEmail(email)) {
			error = 'Please enter a valid email address';
			return;
		}

		isSubmitting = true;

		try {
			await apiClient.forgotPassword(email.trim());
			submitted = true;
		} catch (e: any) {
			error = e?.message || 'Something went wrong. Please try again.';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>Forgot Password - RankChoice</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		{#if submitted}
			<div class="text-center">
				<div class="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-green-100">
					<svg class="h-8 w-8 text-green-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
					</svg>
				</div>
				<h2 class="mt-6 text-2xl font-bold text-gray-900">Check your email</h2>
				<p class="mt-2 text-gray-600">
					If an account with that email exists, we've sent a password reset link. Please check your inbox and spam folder.
				</p>
				<div class="mt-6">
					<a
						href="/login"
						class="font-medium text-indigo-600 hover:text-indigo-500"
					>
						Back to login
					</a>
				</div>
			</div>
		{:else}
			<div>
				<h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
					Reset your password
				</h2>
				<p class="mt-2 text-center text-sm text-gray-600">
					Enter your email address and we'll send you a link to reset your password.
				</p>
			</div>

			<form class="mt-8 space-y-6" onsubmit={handleSubmit}>
				<div>
					<label for="email" class="block text-sm font-medium text-gray-700">
						Email Address
					</label>
					<input
						id="email"
						name="email"
						type="email"
						autocomplete="email"
						required
						class="mt-1 appearance-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-md focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
						class:border-red-300={error.length > 0}
						placeholder="Enter your email address"
						bind:value={email}
						oninput={() => { error = ''; }}
						disabled={isSubmitting}
					/>
				</div>

				{#if error}
					<div class="text-red-600 text-sm">
						<p>{error}</p>
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
							Sending...
						{:else}
							Send reset link
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
