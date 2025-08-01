<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte.js';
	import { apiClient } from '$lib/api/client.js';
	import type { Poll, Candidate, PollResults, RCVRound, Voter, VotersListResponse, CreateVoterRequest } from '$lib/types.js';

	// Get poll ID from URL
	let pollId = $derived($page.params.id);

	// State
	let poll = $state<Poll | null>(null);
	let candidates = $state<Candidate[]>([]);
	let results = $state<PollResults | null>(null);
	let rounds = $state<RCVRound[]>([]);
	let voters = $state<Voter[]>([]);
	let voterStats = $state<{ total: number; votedCount: number; pendingCount: number }>({ total: 0, votedCount: 0, pendingCount: 0 });
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let activeTab = $state<'overview' | 'voters' | 'results'>('overview');
	let votersLoading = $state(false);
	let voterFormData = $state<CreateVoterRequest>({ email: '' });
	let addingVoter = $state(false);
	let voterError = $state<string | null>(null);

	// Redirect if not authenticated (wait for auth to load first)
	$effect(() => {
		if (!authStore.isLoading && !authStore.isAuthenticated) {
			goto('/login');
		}
	});

	// Load poll data
	async function loadPoll() {
		const currentPollId = pollId;
		if (!authStore.isAuthenticated || !currentPollId) return;

		try {
			isLoading = true;
			error = null;

			// Load poll details
			poll = await apiClient.getPoll(currentPollId);
			
			// Load candidates
			candidates = await apiClient.getCandidates(currentPollId);

			// Always load voter stats for the overview
			try {
				const votersData = await apiClient.getVoters(currentPollId);
				voterStats = {
					total: votersData.total,
					votedCount: votersData.votedCount,
					pendingCount: votersData.pendingCount
				};
			} catch (voterError) {
				// If voters endpoint fails, set empty stats
				voterStats = { total: 0, votedCount: 0, pendingCount: 0 };
			}

			// Try to load results (may fail if no votes yet)
			try {
				results = await apiClient.getPollResults(currentPollId);
				const roundsData = await apiClient.getRCVRounds(currentPollId);
				rounds = roundsData.rounds;
			} catch (resultsError) {
				// Results not available yet - this is okay
				results = null;
				rounds = [];
			}

			// Load voters if on voters tab
			if (activeTab === 'voters') {
				await loadVoters();
			}
		} catch (err: any) {
			console.error('Error loading poll:', err);
			if (err.status === 404) {
				error = 'Poll not found';
			} else if (err.status === 403) {
				error = 'You do not have permission to view this poll';
			} else {
				error = 'Failed to load poll. Please try again.';
			}
		} finally {
			isLoading = false;
		}
	}

	// Load voters
	async function loadVoters() {
		if (!pollId || !authStore.isAuthenticated) return;

		try {
			votersLoading = true;
			voterError = null;
			const votersData = await apiClient.getVoters(pollId);
			voters = votersData.voters;
			voterStats = {
				total: votersData.total,
				votedCount: votersData.votedCount,
				pendingCount: votersData.pendingCount
			};
		} catch (e: any) {
			voterError = e.message || 'Failed to load voters';
		} finally {
			votersLoading = false;
		}
	}

	// Add voter
	async function addVoter() {
		if (!pollId || addingVoter) return;

		try {
			addingVoter = true;
			voterError = null;

			await apiClient.createVoter(pollId, voterFormData);
			
			// Clear form
			voterFormData = { email: '' };
			
			// Reload voters and poll data to update all stats
			await Promise.all([
				loadVoters(),
				loadPoll() // This will refresh the main poll stats
			]);
		} catch (e: any) {
			voterError = e.message || 'Failed to add voter';
		} finally {
			addingVoter = false;
		}
	}

	// Handle tab change
	async function handleTabChange(tab: 'overview' | 'voters' | 'results') {
		activeTab = tab;
		if (tab === 'voters' && voters.length === 0) {
			await loadVoters();
		}
	}

	// Copy to clipboard
	async function copyToClipboard(text: string) {
		try {
			await navigator.clipboard.writeText(text);
		} catch (e) {
			// Fallback for older browsers
			const textArea = document.createElement('textarea');
			textArea.value = text;
			document.body.appendChild(textArea);
			textArea.select();
			document.execCommand('copy');
			document.body.removeChild(textArea);
		}
	}

	// Load poll when component mounts or pollId changes
	onMount(loadPoll);

	// Format date
	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
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

	// Copy voting link (placeholder)
	function copyVotingLink() {
		// TODO: Implement when we have voter management
		alert('Voting link functionality will be implemented when voter management is added');
	}

	// Share results (placeholder)
	function shareResults() {
		if (poll) {
			const url = `${window.location.origin}/polls/${poll.id}/results`;
			navigator.clipboard.writeText(url);
			alert('Results link copied to clipboard!');
		}
	}

	// Get winner info
	function getWinner(): string {
		if (!results || !results.winner) return 'No winner determined yet';
		return `${results.winner.name} (${results.winner.percentage.toFixed(1)}%)`;
	}
</script>

<svelte:head>
	<title>{poll ? poll.title : 'Poll Management'} - RankChoice</title>
	<meta name="description" content="Manage your ranked-choice voting poll" />
</svelte:head>

<div class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
	{#if isLoading}
		<!-- Loading state -->
		<div class="text-center py-12">
			<svg class="animate-spin h-8 w-8 text-gray-400 mx-auto" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
				<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
				<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
			</svg>
			<p class="mt-2 text-sm text-gray-500">Loading poll details...</p>
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
							onclick={() => goto('/dashboard')}
							class="bg-red-100 px-2 py-1 text-sm text-red-800 rounded hover:bg-red-200"
						>
							Back to Dashboard
						</button>
					</div>
				</div>
			</div>
		</div>
	{:else if poll}
		<!-- Header -->
		<div class="mb-8">
			<div class="flex items-center mb-4">
				<a href="/dashboard" class="text-indigo-600 hover:text-indigo-500 mr-4">
					<svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
					</svg>
				</a>
				<div class="flex-1">
					<div class="flex items-center">
						<h1 class="text-3xl font-bold text-gray-900">{poll.title}</h1>
						<span class="ml-3 inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium {getStatusColor(getPollStatus(poll))}">
							{getPollStatus(poll)}
						</span>
					</div>
					{#if poll.description}
						<p class="mt-1 text-sm text-gray-500">{poll.description}</p>
					{/if}
				</div>
			</div>

			<!-- Quick Stats -->
			<div class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
				<div class="bg-white overflow-hidden shadow rounded-lg">
					<div class="p-5">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<svg class="h-6 w-6 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 715.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
								</svg>
							</div>
							<div class="ml-5 w-0 flex-1">
								<dl>
									<dt class="text-sm font-medium text-gray-500 truncate">
										Candidates
									</dt>
									<dd class="text-lg font-medium text-gray-900">
										{candidates.length}
									</dd>
								</dl>
							</div>
						</div>
					</div>
				</div>

				<div class="bg-white overflow-hidden shadow rounded-lg" data-testid="voters-stats-card">
					<div class="p-5">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<svg class="h-6 w-6 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
								</svg>
							</div>
							<div class="ml-5 w-0 flex-1">
								<dl>
									<dt class="text-sm font-medium text-gray-500 truncate">
										Voters
									</dt>
									<dd class="text-lg font-medium text-gray-900" data-testid="voters-total-count">
										{voterStats.total}
										{#if voterStats.total > 0}
											<span class="text-sm text-gray-500" data-testid="voters-voted-count">({voterStats.votedCount || 0} voted)</span>
										{/if}
									</dd>
								</dl>
							</div>
						</div>
					</div>
				</div>

				<div class="bg-white overflow-hidden shadow rounded-lg">
					<div class="p-5">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<svg class="h-6 w-6 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
								</svg>
							</div>
							<div class="ml-5 w-0 flex-1">
								<dl>
									<dt class="text-sm font-medium text-gray-500 truncate">
										Total Votes
									</dt>
									<dd class="text-lg font-medium text-gray-900">
										{results?.totalVotes || voterStats.votedCount}
									</dd>
								</dl>
							</div>
						</div>
					</div>
				</div>

				<div class="bg-white overflow-hidden shadow rounded-lg">
					<div class="p-5">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<svg class="h-6 w-6 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2M7 4h10M7 4l-2 14h14l-2-14M11 9v6m2-6v6" />
								</svg>
							</div>
							<div class="ml-5 w-0 flex-1">
								<dl>
									<dt class="text-sm font-medium text-gray-500 truncate">
										Poll Type
									</dt>
									<dd class="text-lg font-medium text-gray-900">
										{poll.pollType === 'single_winner' ? 'Single' : `Multi (${poll.numWinners})`}
									</dd>
								</dl>
							</div>
						</div>
					</div>
				</div>

				<div class="bg-white overflow-hidden shadow rounded-lg">
					<div class="p-5">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<svg class="h-6 w-6 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
								</svg>
							</div>
							<div class="ml-5 w-0 flex-1">
								<dl>
									<dt class="text-sm font-medium text-gray-500 truncate">
										Visibility
									</dt>
									<dd class="text-lg font-medium text-gray-900">
										{poll.isPublic ? 'Public' : 'Private'}
									</dd>
								</dl>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Tabs -->
		<div class="border-b border-gray-200 mb-6">
			<nav class="-mb-px flex space-x-8">
				<button
					data-testid="overview-tab"
					onclick={() => handleTabChange('overview')}
					class="py-2 px-1 border-b-2 font-medium text-sm"
					class:border-indigo-500={activeTab === 'overview'}
					class:text-indigo-600={activeTab === 'overview'}
					class:border-transparent={activeTab !== 'overview'}
					class:text-gray-500={activeTab !== 'overview'}
					class:hover:text-gray-700={activeTab !== 'overview'}
				>
					Overview
				</button>
				<button
					data-testid="voters-tab"
					onclick={() => handleTabChange('voters')}
					class="py-2 px-1 border-b-2 font-medium text-sm"
					class:border-indigo-500={activeTab === 'voters'}
					class:text-indigo-600={activeTab === 'voters'}
					class:border-transparent={activeTab !== 'voters'}
					class:text-gray-500={activeTab !== 'voters'}
					class:hover:text-gray-700={activeTab !== 'voters'}
				>
					Voters
					{#if voterStats.total > 0}
						<span class="ml-2 bg-gray-100 text-gray-900 hidden sm:inline-block py-0.5 px-2.5 rounded-full text-xs font-medium" data-testid="voters-tab-badge">
							{voterStats.total}
						</span>
					{/if}
				</button>
				<button
					data-testid="results-tab"
					onclick={() => handleTabChange('results')}
					class="py-2 px-1 border-b-2 font-medium text-sm"
					class:border-indigo-500={activeTab === 'results'}
					class:text-indigo-600={activeTab === 'results'}
					class:border-transparent={activeTab !== 'results'}
					class:text-gray-500={activeTab !== 'results'}
					class:hover:text-gray-700={activeTab !== 'results'}
				>
					Results
				</button>
			</nav>
		</div>

		<!-- Tab Content -->
		{#if activeTab === 'overview'}
			<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
				<!-- Poll Details -->
				<div class="bg-white shadow rounded-lg p-6">
					<h3 class="text-lg font-medium text-gray-900 mb-4">Poll Details</h3>
					<dl class="space-y-3">
						<div>
							<dt class="text-sm font-medium text-gray-500">Created</dt>
							<dd class="text-sm text-gray-900">{formatDate(poll.createdAt)}</dd>
						</div>
						{#if poll.opensAt}
							<div>
								<dt class="text-sm font-medium text-gray-500">Opens</dt>
								<dd class="text-sm text-gray-900">{formatDate(poll.opensAt)}</dd>
							</div>
						{/if}
						{#if poll.closesAt}
							<div>
								<dt class="text-sm font-medium text-gray-500">Closes</dt>
								<dd class="text-sm text-gray-900">{formatDate(poll.closesAt)}</dd>
							</div>
						{/if}
						<div>
							<dt class="text-sm font-medium text-gray-500">Registration Required</dt>
							<dd class="text-sm text-gray-900">{poll.registrationRequired ? 'Yes' : 'No'}</dd>
						</div>
					</dl>
				</div>

				<!-- Quick Actions -->
				<div class="bg-white shadow rounded-lg p-6">
					<h3 class="text-lg font-medium text-gray-900 mb-4">Quick Actions</h3>
					<div class="space-y-3">
						<button
							onclick={copyVotingLink}
							class="w-full flex items-center justify-center px-4 py-2 border border-gray-300 rounded-md shadow-sm bg-white text-sm font-medium text-gray-700 hover:bg-gray-50"
						>
							<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
							</svg>
							Get Voting Links
						</button>
						{#if results}
							<button
								onclick={shareResults}
								class="w-full flex items-center justify-center px-4 py-2 border border-gray-300 rounded-md shadow-sm bg-white text-sm font-medium text-gray-700 hover:bg-gray-50"
							>
								<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
								</svg>
								Share Results
							</button>
						{/if}
						<button
							onclick={() => poll?.id && goto(`/polls/${poll.id}/edit`)}
							class="w-full flex items-center justify-center px-4 py-2 border border-gray-300 rounded-md shadow-sm bg-white text-sm font-medium text-gray-700 hover:bg-gray-50"
						>
							<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
							</svg>
							Edit Poll
						</button>
					</div>
				</div>
			</div>

			<!-- Candidates List -->
			<div class="mt-6 bg-white shadow rounded-lg">
				<div class="px-6 py-4 border-b border-gray-200">
					<h3 class="text-lg font-medium text-gray-900">Candidates ({candidates.length})</h3>
				</div>
				<ul class="divide-y divide-gray-200">
					{#each candidates as candidate, index}
						<li class="px-6 py-4">
							<div class="flex items-start">
								<span class="bg-indigo-100 text-indigo-800 text-xs font-medium mr-3 px-2.5 py-0.5 rounded-full">{index + 1}</span>
								<div class="flex-1">
									<h4 class="text-sm font-medium text-gray-900">{candidate.name}</h4>
									{#if candidate.description}
										<p class="text-sm text-gray-500 mt-1">{candidate.description}</p>
									{/if}
								</div>
							</div>
						</li>
					{/each}
				</ul>
			</div>
		{:else if activeTab === 'voters'}
			<!-- Voter Management -->
			<div class="space-y-6">
				<!-- Voter Stats -->
				<div class="bg-white shadow rounded-lg">
					<div class="px-6 py-4 border-b border-gray-200">
						<h3 class="text-lg font-medium text-gray-900">Voter Overview</h3>
					</div>
					<div class="px-6 py-4">
						<dl class="grid grid-cols-1 gap-x-4 gap-y-6 sm:grid-cols-3">
							<div>
								<dt class="text-sm font-medium text-gray-500">Total Voters</dt>
								<dd class="mt-1 text-3xl font-semibold text-gray-900" data-testid="voters-total-stat">{voterStats.total || 0}</dd>
							</div>
							<div>
								<dt class="text-sm font-medium text-gray-500">Voted</dt>
								<dd class="mt-1 text-3xl font-semibold text-green-600" data-testid="voters-voted-stat">{voterStats.votedCount || 0}</dd>
							</div>
							<div>
								<dt class="text-sm font-medium text-gray-500">Pending</dt>
								<dd class="mt-1 text-3xl font-semibold text-yellow-600" data-testid="voters-pending-stat">{voterStats.pendingCount || 0}</dd>
							</div>
						</dl>
					</div>
				</div>

				<!-- Add Voter Form -->
				<div class="bg-white shadow rounded-lg">
					<div class="px-6 py-4 border-b border-gray-200">
						<h3 class="text-lg font-medium text-gray-900">Invite Voter</h3>
						<p class="mt-1 text-sm text-gray-500">Generate a unique voting link for a voter</p>
					</div>
					<form onsubmit={e => { e.preventDefault(); addVoter(); }} class="px-6 py-4">
						{#if voterError}
							<div class="mb-4 bg-red-50 border border-red-200 rounded-md p-3">
								<p class="text-sm text-red-600">{voterError}</p>
							</div>
						{/if}
						
						<div class="flex gap-4">
							<div class="flex-1">
								<label for="voter-email" class="block text-sm font-medium text-gray-700">
									Email (optional)
								</label>
								<input
									id="voter-email"
									data-testid="voter-email-input"
									type="email"
									bind:value={voterFormData.email}
									placeholder="voter@example.com"
									class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
									disabled={addingVoter}
								/>
								<p class="mt-1 text-xs text-gray-500">
									Leave empty to generate an anonymous voting link
								</p>
							</div>
							<div class="flex items-end">
								<button
									type="submit"
									data-testid="add-voter-btn"
									disabled={addingVoter}
									class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-300 disabled:cursor-not-allowed"
								>
									{#if addingVoter}
										<svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
											<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
											<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
										</svg>
										Adding...
									{:else}
										<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
										</svg>
										Add Voter
									{/if}
								</button>
							</div>
						</div>
					</form>
				</div>

				<!-- Voters List -->
				<div class="bg-white shadow rounded-lg">
					<div class="px-6 py-4 border-b border-gray-200">
						<h3 class="text-lg font-medium text-gray-900">Voters ({voters.length})</h3>
					</div>
					
					{#if votersLoading}
						<div class="px-6 py-8 text-center">
							<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto"></div>
							<p class="mt-2 text-sm text-gray-500">Loading voters...</p>
						</div>
					{:else if voters.length === 0}
						<div class="px-6 py-8 text-center">
							<svg class="mx-auto h-12 w-12 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
							</svg>
							<h4 class="mt-4 text-lg font-medium text-gray-900">No voters yet</h4>
							<p class="mt-2 text-sm text-gray-500">Add voters to start collecting votes for this poll.</p>
						</div>
					{:else}
						<ul class="divide-y divide-gray-200">
							{#each voters as voter}
								<li class="px-6 py-4">
									<div class="flex items-center justify-between">
										<div class="flex items-center">
											<div class="flex-shrink-0">
												{#if voter.hasVoted}
													<div class="h-8 w-8 bg-green-100 rounded-full flex items-center justify-center">
														<svg class="h-5 w-5 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
														</svg>
													</div>
												{:else}
													<div class="h-8 w-8 bg-yellow-100 rounded-full flex items-center justify-center">
														<svg class="h-5 w-5 text-yellow-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
														</svg>
													</div>
												{/if}
											</div>
											<div class="ml-4">
												<div class="flex items-center">
													<p class="text-sm font-medium text-gray-900">
														{voter.email || `Voter ${voter.ballotToken.slice(-6)}`}
													</p>
													<span class={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
														voter.hasVoted 
															? 'bg-green-100 text-green-800' 
															: 'bg-yellow-100 text-yellow-800'
													}`}>
														{voter.hasVoted ? 'Voted' : 'Pending'}
													</span>
												</div>
												<p class="text-sm text-gray-500">
													{#if voter.hasVoted && voter.votedAt}
														Voted {formatDate(voter.votedAt)}
													{:else}
														Invited {formatDate(voter.invitedAt)}
													{/if}
												</p>
											</div>
										</div>
										<div class="flex items-center space-x-2">
											<button
												onclick={() => copyToClipboard(voter.votingUrl)}
												class="inline-flex items-center px-3 py-1 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
											>
												<svg class="h-4 w-4 mr-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
												</svg>
												Copy Link
											</button>
											<a
												href={voter.votingUrl}
												target="_blank"
												rel="noopener noreferrer"
												class="inline-flex items-center px-3 py-1 border border-transparent text-sm font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
											>
												<svg class="h-4 w-4 mr-1" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
												</svg>
												Open
											</a>
										</div>
									</div>
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			</div>
		{:else if activeTab === 'results'}
			<!-- Results -->
			{#if results && results.totalVotes > 0}
				<div class="space-y-6">
					<!-- Winner Announcement -->
					{#if results.winner}
						<div class="bg-green-50 border border-green-200 rounded-lg p-6">
							<div class="flex">
								<div class="flex-shrink-0">
									<svg class="h-5 w-5 text-green-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
										<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
									</svg>
								</div>
								<div class="ml-3">
									<h3 class="text-lg font-medium text-green-800">
										🏆 Winner: {results.winner.name}
									</h3>
									<p class="mt-1 text-sm text-green-700">
										Won with {results.winner.finalVotes} votes ({results.winner.percentage.toFixed(1)}% of total)
									</p>
								</div>
							</div>
						</div>
					{/if}

					<!-- Final Rankings -->
					<div class="bg-white shadow rounded-lg">
						<div class="px-6 py-4 border-b border-gray-200">
							<h3 class="text-lg font-medium text-gray-900">Final Rankings</h3>
						</div>
						<ul class="divide-y divide-gray-200">
							{#each results.finalRankings as ranking}
								<li class="px-6 py-4">
									<div class="flex items-center justify-between">
										<div class="flex items-center">
											<span class="bg-gray-100 text-gray-800 text-sm font-medium mr-3 px-2.5 py-0.5 rounded-full">
												#{ranking.position}
											</span>
											<div>
												<h4 class="text-sm font-medium text-gray-900">{ranking.name}</h4>
												{#if ranking.eliminatedRound}
													<p class="text-xs text-gray-500">Eliminated in round {ranking.eliminatedRound}</p>
												{/if}
											</div>
										</div>
										<div class="text-right">
											<div class="text-sm font-medium text-gray-900">{ranking.votes} votes</div>
											<div class="text-sm text-gray-500">{ranking.percentage.toFixed(1)}%</div>
										</div>
									</div>
								</li>
							{/each}
						</ul>
					</div>

					<!-- RCV Rounds -->
					{#if rounds.length > 1}
						<div class="bg-white shadow rounded-lg">
							<div class="px-6 py-4 border-b border-gray-200">
								<h3 class="text-lg font-medium text-gray-900">RCV Elimination Rounds</h3>
								<p class="text-sm text-gray-500">Round-by-round vote counting and eliminations</p>
							</div>
							<div class="overflow-x-auto">
								<table class="min-w-full divide-y divide-gray-200">
									<thead class="bg-gray-50">
										<tr>
											<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
												Round
											</th>
											{#each candidates as candidate}
												<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
													{candidate.name}
												</th>
											{/each}
											<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
												Action
											</th>
										</tr>
									</thead>
									<tbody class="bg-white divide-y divide-gray-200">
										{#each rounds as round}
											<tr>
												<td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
													{round.roundNumber}
												</td>
												{#each candidates as candidate}
													<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
														{round.voteCounts[candidate.id] || 0}
													</td>
												{/each}
												<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
													{#if round.winner}
														<span class="text-green-600 font-medium">Winner!</span>
													{:else if round.eliminated}
														<span class="text-red-600">Eliminated: {candidates.find(c => c.id === round.eliminated)?.name}</span>
													{/if}
												</td>
											</tr>
										{/each}
									</tbody>
								</table>
							</div>
						</div>
					{/if}
				</div>
			{:else}
				<!-- No results yet -->
				<div class="bg-gray-50 rounded-lg p-8 text-center">
					<svg class="mx-auto h-12 w-12 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
					</svg>
					<h3 class="mt-4 text-lg font-medium text-gray-900">No Votes Yet</h3>
					<p class="mt-2 text-sm text-gray-500">
						Results will appear here once people start voting. Share your poll to collect votes!
					</p>
				</div>
			{/if}
		{/if}
	{/if}
</div> 