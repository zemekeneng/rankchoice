<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiClient } from '$lib/api/client.js';
	import RCVVisualization from '$lib/components/RCVVisualization.svelte';
	import type { Poll, Candidate, PollResults, RCVRound } from '$lib/types.js';

	// Get poll ID from URL
	let pollId = $derived($page.params.id);

	// State
	let poll = $state<Poll | null>(null);
	let candidates = $state<Candidate[]>([]);
	let results = $state<PollResults | null>(null);
	let rounds = $state<RCVRound[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Load poll data (public endpoint - no auth required)
	async function loadResults() {
		if (!pollId) return;

		try {
			isLoading = true;
			error = null;

			// Load poll details (public endpoint)
			poll = await apiClient.getPoll(pollId);
			
			// Load candidates
			candidates = await apiClient.getCandidates(pollId);

			// Load results
			results = await apiClient.getPollResults(pollId);
			const roundsData = await apiClient.getRCVRounds(pollId);
			rounds = roundsData.rounds;

		} catch (err: any) {
			console.error('Error loading results:', err);
			if (err.status === 404) {
				error = 'Poll not found';
			} else if (err.status === 403) {
				error = 'Results are not yet available for this poll';
			} else {
				error = 'Failed to load results. Please try again.';
			}
		} finally {
			isLoading = false;
		}
	}

	// Load data on mount
	onMount(loadResults);

	// Reload when poll ID changes
	$effect(() => {
		if (pollId) {
			loadResults();
		}
	});

	// Get poll status
	function getPollStatus(): string {
		if (!poll) return 'Unknown';
		const now = new Date();
		const opensAt = poll.opensAt ? new Date(poll.opensAt) : null;
		const closesAt = poll.closesAt ? new Date(poll.closesAt) : null;

		if (opensAt && now < opensAt) {
			return 'Scheduled';
		} else if (closesAt && now > closesAt) {
			return 'Closed';
		} else {
			return 'Active';
		}
	}

	// Get status color
	function getStatusColor(): string {
		const status = getPollStatus();
		switch (status) {
			case 'Active':
				return 'bg-green-100 text-green-800';
			case 'Scheduled':
				return 'bg-yellow-100 text-yellow-800';
			case 'Closed':
				return 'bg-gray-100 text-gray-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	// Share functions
	function shareResults() {
		if (navigator.share && poll) {
			navigator.share({
				title: `${poll.title} - Results`,
				text: `Check out the results for "${poll.title}"`,
				url: window.location.href
			});
		} else {
			// Fallback: copy to clipboard
			navigator.clipboard.writeText(window.location.href);
			alert('Results link copied to clipboard!');
		}
	}

	function copyLink() {
		navigator.clipboard.writeText(window.location.href);
		alert('Results link copied to clipboard!');
	}

	// Format date for display
	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString() + ' at ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	}
</script>

<svelte:head>
	<title>{poll ? `${poll.title} - Results` : 'Poll Results'} - RankChoice</title>
	<meta name="description" content={poll ? `Results for ranked-choice poll: ${poll.title}` : 'Ranked-choice voting poll results'} />
	{#if poll}
		<meta property="og:title" content="{poll.title} - Results" />
		<meta property="og:description" content="View the results of this ranked-choice voting poll" />
		<meta property="og:type" content="website" />
		<meta property="og:url" content={typeof window !== 'undefined' ? window.location.href : ''} />
	{/if}
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Header -->
	<div class="bg-white shadow">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex justify-between items-center py-6">
				<div class="flex items-center">
					<a href="/" class="flex items-center space-x-2 text-indigo-600 hover:text-indigo-500">
						<svg class="h-8 w-8" fill="currentColor" viewBox="0 0 24 24">
							<path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
						</svg>
						<span class="text-xl font-bold">RankChoice</span>
					</a>
				</div>
				
				<div class="flex items-center space-x-4">
					{#if poll}
						<button
							onclick={shareResults}
							class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
						>
							<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
							</svg>
							Share Results
						</button>
						<button
							onclick={copyLink}
							class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
						>
							<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
							</svg>
							Copy Link
						</button>
					{/if}
				</div>
			</div>
		</div>
	</div>

	<!-- Main Content -->
	<main class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
		{#if isLoading}
			<!-- Loading State -->
			<div class="flex justify-center items-center py-12">
				<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
			</div>
		{:else if error}
			<!-- Error State -->
			<div class="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
				<div class="flex justify-center">
					<svg class="h-12 w-12 text-red-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L5.08 16.5c-.77.833.192 2.5 1.732 2.5z" />
					</svg>
				</div>
				<h3 class="mt-4 text-lg font-medium text-red-800">Error Loading Results</h3>
				<p class="mt-2 text-sm text-red-600">{error}</p>
				<div class="mt-4">
					<button
						onclick={loadResults}
						class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
					>
						Try Again
					</button>
				</div>
			</div>
		{:else if poll && results}
			<!-- Results Content -->
			<div class="space-y-6">
				<!-- Poll Header -->
				<div class="bg-white shadow rounded-lg p-6">
					<div class="flex items-start justify-between">
						<div class="flex-1">
							<h1 class="text-3xl font-bold text-gray-900 mb-2">{poll.title}</h1>
							{#if poll.description}
								<p class="text-gray-600 mb-4">{poll.description}</p>
							{/if}
							
							<div class="flex flex-wrap items-center gap-4 text-sm">
								<span class="inline-flex items-center px-3 py-1 rounded-full {getStatusColor()}">
									{getPollStatus()}
								</span>
								<span class="inline-flex items-center px-3 py-1 rounded-full bg-indigo-100 text-indigo-800">
									{poll.pollType === 'single_winner' ? 'Single Winner' : `${poll.numWinners} Winners`}
								</span>
								<span class="text-gray-500">
									{results.totalVotes} total votes
								</span>
								{#if poll.closesAt}
									<span class="text-gray-500">
										Closed {formatDate(poll.closesAt)}
									</span>
								{/if}
							</div>
						</div>
					</div>
				</div>

				<!-- Winner Announcement -->
				{#if results.winner}
					<div class="bg-gradient-to-r from-green-50 to-emerald-50 border border-green-200 rounded-lg p-6">
						<div class="flex items-center">
							<div class="flex-shrink-0">
								<div class="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center">
									<svg class="w-6 h-6 text-green-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
									</svg>
								</div>
							</div>
							<div class="ml-4">
								<h2 class="text-2xl font-bold text-green-800">
									🏆 Winner: {results.winner.name}
								</h2>
								<p class="text-green-700">
									Won with {results.winner.finalVotes} votes ({results.winner.percentage.toFixed(1)}% of total)
								</p>
								<p class="text-sm text-green-600 mt-1">
									Achieved majority through ranked-choice voting
								</p>
							</div>
						</div>
					</div>
				{/if}

				<!-- Final Rankings -->
				<div class="bg-white shadow rounded-lg">
					<div class="px-6 py-4 border-b border-gray-200">
						<h3 class="text-lg font-medium text-gray-900">Final Rankings</h3>
						<p class="text-sm text-gray-500">Complete results after all RCV rounds</p>
					</div>
					<ul class="divide-y divide-gray-200">
						{#each results.finalRankings as ranking}
							<li class="px-6 py-4">
								<div class="flex items-center justify-between">
									<div class="flex items-center">
										<div class="flex-shrink-0 mr-4">
											{#if ranking.position === 1}
												<div class="w-8 h-8 bg-yellow-100 text-yellow-800 rounded-full flex items-center justify-center text-sm font-bold">
													🥇
												</div>
											{:else if ranking.position === 2}
												<div class="w-8 h-8 bg-gray-100 text-gray-800 rounded-full flex items-center justify-center text-sm font-bold">
													🥈
												</div>
											{:else if ranking.position === 3}
												<div class="w-8 h-8 bg-orange-100 text-orange-800 rounded-full flex items-center justify-center text-sm font-bold">
													🥉
												</div>
											{:else}
												<div class="w-8 h-8 bg-gray-100 text-gray-800 rounded-full flex items-center justify-center text-sm font-bold">
													#{ranking.position}
												</div>
											{/if}
										</div>
										<div>
											<h4 class="text-lg font-medium text-gray-900">{ranking.name}</h4>
											{#if ranking.eliminatedRound}
												<p class="text-sm text-gray-500">Eliminated in round {ranking.eliminatedRound}</p>
											{/if}
										</div>
									</div>
									<div class="text-right">
										<div class="text-lg font-semibold text-gray-900">{ranking.votes} votes</div>
										<div class="text-sm text-gray-500">{ranking.percentage.toFixed(1)}%</div>
									</div>
								</div>
							</li>
						{/each}
					</ul>
				</div>

				<!-- Enhanced RCV Visualization -->
				{#if rounds.length > 0}
					<RCVVisualization 
						{rounds} 
						{candidates} 
						totalVotes={results.totalVotes}
					/>
				{/if}

				<!-- About RCV -->
				<div class="bg-blue-50 border border-blue-200 rounded-lg p-6">
					<h3 class="text-lg font-medium text-blue-900 mb-2">About Ranked-Choice Voting</h3>
					<p class="text-blue-800 text-sm">
						Ranked-choice voting allows voters to rank candidates in order of preference. 
						If no candidate receives a majority of first-choice votes, the candidate with the fewest votes is eliminated 
						and their votes are redistributed to voters' next choices. This process continues until a candidate achieves a majority.
					</p>
				</div>

				<!-- Footer -->
				<div class="text-center py-8">
					<p class="text-gray-500 text-sm">
						Powered by 
						<a href="/" class="text-indigo-600 hover:text-indigo-500 font-medium">RankChoice</a>
						- Create your own ranked-choice voting poll
					</p>
				</div>
			</div>
		{/if}
	</main>
</div>