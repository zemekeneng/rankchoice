<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { authStore } from '$lib/stores/auth.svelte.js';
	import { goto } from '$app/navigation';

	let { children } = $props();

	// Logout handler
	async function handleLogout() {
		try {
			await authStore.logout();
		} catch (error) {
			console.error('Logout error:', error);
		}
	}

	// Navigate to dashboard
	function goToDashboard() {
		goto('/dashboard');
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Navigation -->
	<nav class="bg-white shadow-lg">
		<div class="max-w-7xl mx-auto px-4">
			<div class="flex justify-between h-16">
				<div class="flex items-center">
					<a href="/" class="flex-shrink-0 flex items-center">
						<h1 class="text-xl font-bold text-gray-900">RankChoice</h1>
					</a>
				</div>

				<div class="flex items-center space-x-4">
					{#if authStore.isAuthenticated}
						<!-- Authenticated user menu -->
						<span class="text-gray-700">
							Welcome, {authStore.displayName}
						</span>
						<button
							onclick={goToDashboard}
							class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
						>
							Dashboard
						</button>
						<button
							onclick={handleLogout}
							class="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-md text-sm font-medium"
						>
							Logout
						</button>
					{:else}
						<!-- Guest menu -->
						<a
							href="/login"
							class="text-gray-500 hover:text-gray-700 px-3 py-2 rounded-md text-sm font-medium"
						>
							Login
						</a>
						<a
							href="/register"
							class="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium"
						>
							Sign Up
						</a>
					{/if}
				</div>
			</div>
		</div>
	</nav>

	<!-- Main content -->
	<main>
		{@render children?.()}
	</main>
</div>
