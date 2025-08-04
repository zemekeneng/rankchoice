<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiClient } from '$lib/api/client.js';
	import type { Poll, Candidate } from '$lib/types.js';

	// Get poll ID from URL
	let pollId = $derived($page.params.id);

	// State
	let poll = $state<Poll | null>(null);
	let candidates = $state<Candidate[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Load public poll data
	async function loadPublicPoll() {
		if (!pollId) return;

		try {
			isLoading = true;
			error = null;

			// Load public poll details
			const pollData = await apiClient.getPublicPoll(pollId);
			poll = pollData;
			candidates = pollData.candidates || [];

		} catch (err: any) {
			console.error('Error loading public poll:', err);
			if (err.status === 404) {
				error = 'Poll not found';
			} else if (err.status === 403) {
				error = 'This poll is not public';
			} else {
				error = 'Failed to load poll. Please try again.';
			}
		} finally {
			isLoading = false;
		}
	}

	// Load poll on mount
	onMount(() => {
		loadPublicPoll();
	});

	// Format date
	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Check if poll is open for voting
	function isPollOpen(): boolean {
		if (!poll) return false;
		
		const now = new Date();
		const opensAt = poll.opensAt ? new Date(poll.opensAt) : null;
		const closesAt = poll.closesAt ? new Date(poll.closesAt) : null;
		
		if (opensAt && now < opensAt) return false;
		if (closesAt && now > closesAt) return false;
		
		return true;
	}

	// Get poll status
	function getPollStatus(): string {
		if (!poll) return 'Unknown';
		
		const now = new Date();
		const opensAt = poll.opensAt ? new Date(poll.opensAt) : null;
		const closesAt = poll.closesAt ? new Date(poll.closesAt) : null;
		
		if (opensAt && now < opensAt) return 'Opening Soon';
		if (closesAt && now > closesAt) return 'Closed';
		if (isPollOpen()) return 'Open for Voting';
		
		return 'Active';
	}

	// Get status color
	function getStatusColor(status: string): string {
		switch (status) {
			case 'Open for Voting':
			case 'Active':
				return 'bg-green-100 text-green-800';
			case 'Opening Soon':
				return 'bg-yellow-100 text-yellow-800';
			case 'Closed':
				return 'bg-red-100 text-red-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	// Voting state
	let showVotingForm = $state(false);
	let rankedCandidates = $state<Array<{id: string; name: string; description?: string; rank: number}>>([]);
	let unrankedCandidates = $state<Array<{id: string; name: string; description?: string}>>([]);
	let isSubmitting = $state(false);
	let voteSubmitted = $state(false);
	let voteReceipt = $state<{receipt_code: string; verification_url: string} | null>(null);
	let votingError = $state<string | null>(null);

	// Initialize candidates for voting
	function initializeVoting() {
		if (poll?.registrationRequired) {
			alert('This poll requires voter registration. Please contact the poll organizer for a voting link.');
			return;
		}
		
		showVotingForm = true;
		rankedCandidates = [];
		unrankedCandidates = candidates.map(c => ({
			id: c.id,
			name: c.name,
			description: c.description
		}));
		votingError = null;
	}

	// Add candidate to ranking
	function rankCandidate(candidate: {id: string; name: string; description?: string}) {
		const newRank = rankedCandidates.length + 1;
		rankedCandidates = [...rankedCandidates, { ...candidate, rank: newRank }];
		unrankedCandidates = unrankedCandidates.filter(c => c.id !== candidate.id);
	}

	// Remove candidate from ranking
	function unrankCandidate(candidate: {id: string; name: string; description?: string; rank: number}) {
		unrankedCandidates = [...unrankedCandidates, { id: candidate.id, name: candidate.name, description: candidate.description }];
		rankedCandidates = rankedCandidates
			.filter(c => c.id !== candidate.id)
			.map((c, index) => ({ ...c, rank: index + 1 }));
	}

	// Submit anonymous vote
	async function submitVote() {
		if (!poll || rankedCandidates.length === 0) {
			votingError = 'Please rank at least one candidate';
			return;
		}

		try {
			isSubmitting = true;
			votingError = null;

			const rankings = rankedCandidates.map(c => ({
				candidateId: c.id,
				rank: c.rank
			}));

			const result = await apiClient.submitAnonymousVote(poll.id, rankings);
			
			voteReceipt = result.receipt;
			voteSubmitted = true;
			showVotingForm = false;

		} catch (err: any) {
			console.error('Error submitting vote:', err);
			if (err.response?.data?.error?.message) {
				votingError = err.response.data.error.message;
			} else {
				votingError = 'Failed to submit vote. Please try again.';
			}
		} finally {
			isSubmitting = false;
		}
	}

	// Cancel voting
	function cancelVoting() {
		showVotingForm = false;
		rankedCandidates = [];
		unrankedCandidates = [];
		votingError = null;
	}

	// Share functionality
	async function sharePoll() {
		if (!poll) return;

		const shareData = {
			title: `Vote in: ${poll.title}`,
			text: poll.description || `Cast your vote in this ranked choice poll: ${poll.title}`,
			url: window.location.href
		};

		try {
			if (navigator.share) {
				await navigator.share(shareData);
			} else {
				// Fallback: copy to clipboard
				await navigator.clipboard.writeText(window.location.href);
				alert('Poll link copied to clipboard!');
			}
		} catch (err) {
			console.error('Error sharing:', err);
			// Fallback: copy to clipboard
			try {
				await navigator.clipboard.writeText(window.location.href);
				alert('Poll link copied to clipboard!');
			} catch (clipboardErr) {
				console.error('Clipboard error:', clipboardErr);
				alert('Unable to share or copy link. Please copy the URL manually.');
			}
		}
	}
</script>

<svelte:head>
	<title>{poll ? `Vote in: ${poll.title}` : 'Public Poll'} - RankChoice</title>
	<meta name="description" content={poll ? (poll.description || `Cast your vote in this ranked choice poll: ${poll.title}`) : 'Participate in ranked choice voting'} />
	{#if poll}
		<meta property="og:title" content="Vote in: {poll.title}" />
		<meta property="og:description" content={poll.description || `Cast your vote in this ranked choice poll`} />
		<meta property="og:type" content="website" />
		<meta property="og:url" content="{$page.url.href}" />
		<meta name="twitter:card" content="summary" />
		<meta name="twitter:title" content="Vote in: {poll.title}" />
		<meta name="twitter:description" content={poll.description || `Cast your vote in this ranked choice poll`} />
	{/if}
</svelte:head>

<div class="min-h-screen bg-gray-50">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-screen">
			<div class="text-center">
				<div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600 mx-auto mb-4"></div>
				<p class="text-gray-600">Loading poll...</p>
			</div>
		</div>
	{:else if error}
		<div class="max-w-4xl mx-auto py-8 px-4">
			<div class="bg-red-50 border border-red-200 rounded-md p-4">
				<h2 class="text-red-800 font-semibold">Error Loading Poll</h2>
				<p class="text-red-600">{error}</p>
				<button 
					onclick={() => window.location.href = '/'}
					class="mt-2 text-red-700 underline hover:text-red-800"
				>
					← Go to Homepage
				</button>
			</div>
		</div>
	{:else if poll}
		<div class="max-w-4xl mx-auto py-8 px-4">
			<!-- Header -->
			<div class="bg-white shadow rounded-lg p-6 mb-6">
				<div class="flex items-start justify-between">
					<div class="flex-1">
						<div class="flex items-center mb-2">
							<h1 class="text-3xl font-bold text-gray-900">{poll.title}</h1>
							<span class="ml-3 inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium {getStatusColor(getPollStatus())}">
								{getPollStatus()}
							</span>
						</div>
						{#if poll.description}
							<p class="text-gray-600 mb-4">{poll.description}</p>
						{/if}

						<!-- Poll Details -->
						<div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-500">
							<div>
								<span class="font-medium">Poll Type:</span>
								{poll.pollType === 'single_winner' ? 'Single Winner' : `${poll.numWinners} Winners`}
							</div>
							<div>
								<span class="font-medium">Candidates:</span>
								{candidates.length}
							</div>
							{#if poll.opensAt}
								<div>
									<span class="font-medium">Opens:</span>
									{formatDate(poll.opensAt)}
								</div>
							{/if}
							{#if poll.closesAt}
								<div>
									<span class="font-medium">Closes:</span>
									{formatDate(poll.closesAt)}
								</div>
							{/if}
						</div>
					</div>

					<!-- Action Buttons -->
					<div class="flex space-x-3 ml-4">
						<button
							onclick={sharePoll}
							class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
						>
							<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z" />
							</svg>
							Share Poll
						</button>

						{#if isPollOpen()}
							<button
								onclick={initializeVoting}
								class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
							>
								<svg class="h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
								</svg>
								Vote Now
							</button>
						{/if}
					</div>
				</div>
			</div>

			<!-- Candidates -->
			<div class="bg-white shadow rounded-lg">
				<div class="px-6 py-4 border-b border-gray-200">
					<h2 class="text-lg font-medium text-gray-900">Candidates</h2>
					<p class="text-sm text-gray-500 mt-1">
						{poll.pollType === 'single_winner' 
							? 'Rank these candidates in order of preference' 
							: `Choose your top ${poll.numWinners} candidates in order of preference`}
					</p>
				</div>
				<div class="divide-y divide-gray-200">
					{#each candidates as candidate}
						<div class="px-6 py-4">
							<div class="flex items-start">
								<div class="flex-1">
									<h3 class="text-lg font-medium text-gray-900">{candidate.name}</h3>
									{#if candidate.description}
										<p class="text-gray-600 mt-1">{candidate.description}</p>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- Voting Instructions -->
			{#if isPollOpen()}
				<div class="mt-6 bg-blue-50 border border-blue-200 rounded-lg p-6">
					<div class="flex">
						<svg class="h-6 w-6 text-blue-400 mt-0.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
						<div class="ml-3">
							<h3 class="text-lg font-medium text-blue-900">Ready to Vote?</h3>
							<div class="mt-2 text-sm text-blue-800">
								<p class="mb-2">This is a ranked choice voting poll. When you vote, you'll rank the candidates in order of preference.</p>
								{#if poll.registrationRequired}
									<p class="font-medium">This poll requires voter registration. Contact the poll organizer to get your personal voting link.</p>
								{:else}
									<p>Click "Vote Now" to cast your ballot in this poll.</p>
								{/if}
							</div>
						</div>
					</div>
				</div>
			{:else}
				<div class="mt-6 bg-gray-50 border border-gray-200 rounded-lg p-6">
					<div class="text-center">
						<h3 class="text-lg font-medium text-gray-900">
							{getPollStatus() === 'Closed' ? 'Voting Has Ended' : 'Voting Not Yet Open'}
						</h3>
						<p class="mt-2 text-sm text-gray-600">
							{#if getPollStatus() === 'Closed'}
								This poll is no longer accepting votes.
							{:else if poll.opensAt}
								Voting will open on {formatDate(poll.opensAt)}.
							{:else}
								Voting has not yet begun.
							{/if}
						</p>
					</div>
				</div>
			{/if}

			<!-- Voting Form Modal -->
			{#if showVotingForm}
				<div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
					<div class="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-y-auto">
						<div class="px-6 py-4 border-b border-gray-200">
							<div class="flex items-center justify-between">
								<h3 class="text-lg font-medium text-gray-900">Vote in: {poll?.title}</h3>
								<button onclick={cancelVoting} class="text-gray-400 hover:text-gray-600">
									<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
									</svg>
								</button>
							</div>
						</div>

						<div class="px-6 py-4">
							{#if votingError}
								<div class="mb-4 bg-red-50 border border-red-200 rounded-md p-4">
									<div class="flex">
										<svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
											<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
										</svg>
										<div class="ml-3">
											<p class="text-sm text-red-800">{votingError}</p>
										</div>
									</div>
								</div>
							{/if}

							<p class="text-sm text-gray-600 mb-4">
								Rank the candidates in order of preference. Your first choice should be ranked #1, your second choice #2, and so on.
							</p>

							<!-- Ranked Candidates -->
							{#if rankedCandidates.length > 0}
								<div class="mb-6">
									<h4 class="text-sm font-medium text-gray-900 mb-3">Your Rankings</h4>
									<div class="space-y-2">
										{#each rankedCandidates as candidate}
											<div class="bg-blue-50 border border-blue-200 rounded-lg p-3 flex items-center justify-between">
												<div class="flex items-center">
													<span class="bg-blue-600 text-white text-xs font-bold px-2 py-1 rounded-full mr-3">
														#{candidate.rank}
													</span>
													<div>
														<h5 class="font-medium text-gray-900">{candidate.name}</h5>
														{#if candidate.description}
															<p class="text-sm text-gray-600">{candidate.description}</p>
														{/if}
													</div>
												</div>
												<button
													onclick={() => unrankCandidate(candidate)}
													class="text-red-600 hover:text-red-800"
													title="Remove from ranking"
												>
													<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
														<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
													</svg>
												</button>
											</div>
										{/each}
									</div>
								</div>
							{/if}

							<!-- Unranked Candidates -->
							{#if unrankedCandidates.length > 0}
								<div class="mb-6">
									<h4 class="text-sm font-medium text-gray-900 mb-3">Available Candidates</h4>
									<div class="space-y-2">
										{#each unrankedCandidates as candidate}
											<div class="bg-gray-50 border border-gray-200 rounded-lg p-3 flex items-center justify-between">
												<div>
													<h5 class="font-medium text-gray-900">{candidate.name}</h5>
													{#if candidate.description}
														<p class="text-sm text-gray-600">{candidate.description}</p>
													{/if}
												</div>
												<button
													onclick={() => rankCandidate(candidate)}
													class="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded text-sm font-medium"
												>
													Rank #{rankedCandidates.length + 1}
												</button>
											</div>
										{/each}
									</div>
								</div>
							{/if}
						</div>

						<div class="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3">
							<button
								onclick={cancelVoting}
								class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
							>
								Cancel
							</button>
							<button
								onclick={submitVote}
								disabled={rankedCandidates.length === 0 || isSubmitting}
								class="px-4 py-2 bg-blue-600 border border-transparent rounded-md text-sm font-medium text-white hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
							>
								{#if isSubmitting}
									<svg class="animate-spin h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
										<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
										<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
									</svg>
									Submitting...
								{:else}
									Submit Vote
								{/if}
							</button>
						</div>
					</div>
				</div>
			{/if}

			<!-- Vote Success Message -->
			{#if voteSubmitted && voteReceipt}
				<div class="mt-6 bg-green-50 border border-green-200 rounded-lg p-6">
					<div class="flex">
						<svg class="h-6 w-6 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
						<div class="ml-3">
							<h3 class="text-lg font-medium text-green-900">Vote Submitted Successfully!</h3>
							<div class="mt-2 text-sm text-green-800">
								<p class="mb-2">Thank you for participating in this poll. Your vote has been recorded.</p>
								<div class="bg-white border border-green-200 rounded p-3">
									<p class="font-medium">Receipt Code: <span class="font-mono">{voteReceipt.receipt_code}</span></p>
									<p class="text-xs text-green-600 mt-1">Save this code to verify your vote was counted</p>
								</div>
							</div>
						</div>
					</div>
				</div>
			{/if}

			<!-- Footer -->
			<div class="mt-8 text-center text-sm text-gray-500">
				<p>Powered by <a href="/" class="text-blue-600 hover:text-blue-500">RankChoice</a> - Secure, transparent, democratic voting</p>
			</div>
		</div>
	{/if}
</div>