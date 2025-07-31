<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { authStore } from '$lib/stores/auth.svelte.js';
	import { apiClient } from '$lib/api/client.js';
	import type { Poll } from '$lib/types.js';

	// State
	let polls = $state<Poll[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let showSuccess = $state(false);

	// Redirect if not authenticated (wait for auth to load first)
	$effect(() => {
		if (!authStore.isLoading && !authStore.isAuthenticated) {
			goto('/login');
		}
	});

	// Load user's polls
	async function loadPolls() {
		if (!authStore.isAuthenticated) return;

		try {
			isLoading = true;
			error = null;

			const response = await apiClient.getPolls({
				page: 1,
				limit: 20,
				sort: 'created_at',
				order: 'desc'
			});

			polls = response.polls || [];
		} catch (err) {
			console.error('Error loading polls:', err);
			error = 'Failed to load polls. Please try again.';
			polls = []; // Ensure polls is empty array on error
		} finally {
			isLoading = false;
		}
	}

	// Load polls when component mounts or page URL changes
	$effect(() => {
		if (authStore.isAuthenticated && !authStore.isLoading) {
			loadPolls();
		}
	});

	// Check for success message and clean URL
	$effect(() => {
		if ($page.url.searchParams.get('created') === 'true') {
			showSuccess = true;
			// Clean URL after showing success
			const url = new URL($page.url);
			url.searchParams.delete('created');
			goto(url.pathname + url.search, { replaceState: true });
			
			// Hide success message after 5 seconds
			setTimeout(() => {
				showSuccess = false;
			}, 5000);
		}
	});

	// Navigate to create poll page
	function createPoll() {
		goto('/polls/new', { invalidateAll: true });
	}

	// Navigate to poll details
	function viewPoll(pollId: string) {
		goto(`/polls/${pollId}`);
	}

	// Format date
	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	// Get poll status
	function getPollStatus(poll: Poll): string {
		const now = new Date();
		const opensAt = poll.opensAt ? new Date(poll.opensAt) : null;
		const closesAt = poll.closesAt ? new Date(poll.closesAt) : null;

		if (opensAt && now < opensAt) {
			return 'scheduled';
		} else if (closesAt && now > closesAt) {
			return 'closed';
		} else {
			return 'active';
		}
	}

	// Get status color
	function getStatusColor(status: string): string {
		switch (status) {
			case 'active':
				return 'bg-green-100 text-green-800';
			case 'scheduled':
				return 'bg-yellow-100 text-yellow-800';
			case 'closed':
				return 'bg-gray-100 text-gray-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}
</script>

<svelte:head>
	<title>Dashboard - RankChoice</title>
	<meta name="description" content="Manage your ranked-choice voting polls" />
</svelte:head>

<div class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
	<!-- Header -->
	<div class="md:flex md:items-center md:justify-between">
		<div class="flex-1 min-w-0">
			<h2 class="text-2xl font-bold leading-7 text-gray-900 sm:text-3xl sm:truncate">
				Dashboard
			</h2>
			<p class="mt-1 text-sm text-gray-500">
				Manage your ranked-choice voting polls
			</p>
		</div>
		<div class="mt-4 flex md:mt-0 md:ml-4">
			<button
				onclick={createPoll}
				class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
			>
				<svg class="-ml-1 mr-2 h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
					<path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
				</svg>
				Create Poll
			</button>
		</div>
	</div>

	<!-- Success Message -->
	{#if showSuccess}
		<div class="mb-6 bg-green-50 border border-green-200 rounded-md p-4">
			<div class="flex">
				<div class="flex-shrink-0">
					<svg class="h-5 w-5 text-green-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
					</svg>
				</div>
				<div class="ml-3">
					<h3 class="text-sm font-medium text-green-800">Poll Created Successfully!</h3>
					<div class="mt-2 text-sm text-green-700">
						<p>Your new poll has been created and is ready for voting.</p>
					</div>
				</div>
				<div class="ml-auto pl-3">
					<div class="-mx-1.5 -my-1.5">
						<button
							onclick={() => showSuccess = false}
							class="inline-flex bg-green-50 rounded-md p-1.5 text-green-500 hover:bg-green-100"
						>
							<svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
								<path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
							</svg>
						</button>
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Content -->
	<div class="mt-8">
		{#if isLoading}
			<!-- Loading state -->
			<div class="text-center py-12">
				<svg class="animate-spin h-8 w-8 text-gray-400 mx-auto" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
				<p class="mt-2 text-sm text-gray-500">Loading your polls...</p>
			</div>
		{:else if error}
			<!-- Error state -->
			<div class="bg-red-50 border border-red-200 rounded-md p-4">
				<div class="flex">
					<div class="flex-shrink-0">
						<svg class="h-5 w-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
						</svg>
					</div>
					<div class="ml-3">
						<h3 class="text-sm font-medium text-red-800">Error</h3>
						<div class="mt-2 text-sm text-red-700">
							<p>{error}</p>
						</div>
						<div class="mt-4">
							<button
								onclick={loadPolls}
								class="bg-red-100 px-2 py-1 text-sm text-red-800 rounded hover:bg-red-200"
							>
								Try again
							</button>
						</div>
					</div>
				</div>
			</div>
		{:else if polls.length === 0}
			<!-- Empty state - Onboarding -->
			<div class="text-center py-12">
				<div class="mx-auto h-24 w-24 text-indigo-600 mb-4">
					<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<h3 class="text-lg font-medium text-gray-900">Welcome to RankChoice!</h3>
				<p class="mt-2 text-sm text-gray-600 max-w-md mx-auto">
					Create your first ranked-choice voting poll to get started. It's easy: add candidates, 
					distribute voting links, and let voters rank their preferences.
				</p>
				
				<!-- Feature highlights -->
				<div class="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-3 max-w-2xl mx-auto">
					<div class="text-center">
						<div class="mx-auto h-8 w-8 text-indigo-600 mb-2">
							<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
							</svg>
						</div>
						<h4 class="text-sm font-medium text-gray-900">Add Candidates</h4>
						<p class="text-xs text-gray-500">Add the options voters will rank</p>
					</div>
					<div class="text-center">
						<div class="mx-auto h-8 w-8 text-indigo-600 mb-2">
							<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
							</svg>
						</div>
						<h4 class="text-sm font-medium text-gray-900">Share & Vote</h4>
						<p class="text-xs text-gray-500">Send links to voters</p>
					</div>
					<div class="text-center">
						<div class="mx-auto h-8 w-8 text-indigo-600 mb-2">
							<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
							</svg>
						</div>
						<h4 class="text-sm font-medium text-gray-900">See Results</h4>
						<p class="text-xs text-gray-500">Watch votes get counted</p>
					</div>
				</div>

				<div class="mt-8">
					<button
						onclick={createPoll}
						class="inline-flex items-center px-6 py-3 border border-transparent shadow-sm text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors"
					>
						<svg class="-ml-1 mr-2 h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
						</svg>
						Create Your First Poll
					</button>
				</div>
			</div>
		{:else}
			<!-- Polls list -->
			<div class="bg-white shadow overflow-hidden sm:rounded-md">
				<ul class="divide-y divide-gray-200">
					{#each polls as poll (poll.id)}
						<li>
							<button
								onclick={() => viewPoll(poll.id)}
								class="block hover:bg-gray-50 w-full text-left"
							>
								<div class="px-4 py-4 sm:px-6">
									<div class="flex items-center justify-between">
										<div class="flex items-center">
											<p class="text-sm font-medium text-indigo-600 truncate">
												{poll.title}
											</p>
											<span class="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusColor(getPollStatus(poll))}">
												{getPollStatus(poll)}
											</span>
										</div>
										<div class="ml-2 flex-shrink-0 flex">
											<p class="text-sm text-gray-500">
												{poll.pollType === 'single_winner' ? 'Single Winner' : `Multi Winner (${poll.numWinners})`}
											</p>
										</div>
									</div>
									<div class="mt-2 sm:flex sm:justify-between">
										<div class="sm:flex">
											<p class="flex items-center text-sm text-gray-500">
												{#if poll.description}
													{poll.description}
												{:else}
													No description
												{/if}
											</p>
										</div>
										<div class="mt-2 flex items-center text-sm text-gray-500 sm:mt-0">
											<p>
												Created {formatDate(poll.createdAt)}
											</p>
										</div>
									</div>
								</div>
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
</div> 