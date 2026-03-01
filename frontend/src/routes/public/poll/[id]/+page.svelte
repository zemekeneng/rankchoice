<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { dndzone } from 'svelte-dnd-action';
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
	let rankedCandidates = $state<Array<{id: string; name: string; description?: string; rank?: number}>>([]);
	let unrankedCandidates = $state<Array<{id: string; name: string; description?: string; rank?: number}>>([]);
	let isSubmitting = $state(false);
	let voteSubmitted = $state(false);
	let voteReceipt = $state<{receipt_code: string; verification_url: string} | null>(null);
	let votingError = $state<string | null>(null);

	// Initialize candidates for voting (match vote page structure with _drag suffix for dndzone)
	function initializeVoting() {
		if (poll?.registrationRequired) {
			alert('This poll requires voter registration. Please contact the poll organizer for a voting link.');
			return;
		}
		
		showVotingForm = true;
		rankedCandidates = [];
		unrankedCandidates = candidates.map(c => ({
			id: c.id + '_drag',
			name: c.name,
			description: c.description
		}));
		votingError = null;
	}

	// DnD handlers (match vote page)
	function handleRankedDrop(event: CustomEvent) {
		rankedCandidates = event.detail.items;
		updateRanks();
	}

	function handleUnrankedDrop(event: CustomEvent) {
		unrankedCandidates = event.detail.items;
	}

	function updateRanks() {
		rankedCandidates = rankedCandidates.map((c, index) => ({
			...c,
			rank: index + 1
		}));
	}

	// Add candidate to ranking
	function rankCandidate(candidate: {id: string; name: string; description?: string}) {
		unrankedCandidates = unrankedCandidates.filter(c => c.id !== candidate.id);
		rankedCandidates = [...rankedCandidates, { ...candidate, rank: rankedCandidates.length + 1 }];
	}

	// Remove candidate from ranking
	function unrankCandidate(candidate: {id: string; name: string; description?: string; rank?: number}) {
		rankedCandidates = rankedCandidates.filter(c => c.id !== candidate.id);
		unrankedCandidates = [...unrankedCandidates, { ...candidate, rank: undefined }];
		updateRanks();
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
				candidateId: c.id.replace('_drag', ''),
				rank: c.rank ?? 0
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
								data-testid="start-voting-btn"
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

			<!-- Voting Form Modal (matches /vote/[token] UI) -->
			{#if showVotingForm}
				<div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50 overflow-y-auto">
					<div class="bg-white rounded-lg shadow-xl max-w-4xl w-full my-8 max-h-[90vh] overflow-y-auto">
						<!-- Header (matches vote page) -->
						<div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
							<div>
								<h1 class="text-2xl font-bold text-gray-900">Vote in: {poll?.title}</h1>
								{#if poll?.description}
									<p class="mt-2 text-gray-600">{poll.description}</p>
								{/if}
								<div class="mt-3 flex flex-wrap gap-2 text-sm text-gray-500">
									<span class="inline-flex items-center px-2.5 py-0.5 rounded-full bg-indigo-100 text-indigo-800">
										{poll?.pollType === 'single_winner' ? 'Single Winner' : `${poll?.numWinners ?? 1} Winners`}
									</span>
									<span>Ranked Choice Voting</span>
								</div>
							</div>
							<button onclick={cancelVoting} class="text-gray-400 hover:text-gray-600 p-1" aria-label="Close">
								<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
								</svg>
							</button>
						</div>

						<div class="p-6">
							<!-- Instructions (matches vote page) -->
							<div class="mb-6">
								<h2 class="text-lg font-medium text-gray-900 mb-2">Instructions</h2>
								<div class="bg-blue-50 border border-blue-200 rounded-md p-4">
									<p class="text-sm text-blue-800">
										Drag and drop candidates to rank them in order of your preference. Your #1 choice should be at the top.
										You can rank as many or as few candidates as you like.
									</p>
								</div>
							</div>

							{#if votingError}
								<div class="mb-4 bg-red-50 border border-red-200 rounded-md p-3">
									<p class="text-sm text-red-600">{votingError}</p>
								</div>
							{/if}

							<!-- Two-column layout (matches vote page) -->
							<div class="lg:grid lg:grid-cols-2 lg:gap-6">
								<!-- Ranked Candidates (green zone) -->
								<div class="mb-6 lg:mb-0">
									<h3 class="text-md font-medium text-gray-900 mb-3">Your Rankings</h3>
									<div 
										class="min-h-32 bg-green-50 border-2 border-green-200 border-dashed rounded-lg p-4"
										use:dndzone={{
											items: rankedCandidates,
											flipDurationMs: 200,
											dragDisabled: isSubmitting
										}}
										onconsider={handleRankedDrop}
										onfinalize={handleRankedDrop}
									>
										{#if rankedCandidates.length === 0}
											<p class="text-green-600 text-center py-8">Drop candidates here to rank them</p>
										{:else}
											{#each rankedCandidates as candidate (candidate.id)}
												<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-3 mb-2 cursor-move hover:shadow-md transition-shadow">
													<div class="flex items-center justify-between">
														<div class="flex items-center space-x-3">
															<div class="flex-shrink-0 w-8 h-8 bg-green-100 rounded-full flex items-center justify-center">
																<span class="text-sm font-medium text-green-800">#{candidate.rank}</span>
															</div>
															<div>
																<h4 class="text-sm font-medium text-gray-900">{candidate.name}</h4>
																{#if candidate.description}
																	<p class="text-xs text-gray-500">{candidate.description}</p>
																{/if}
															</div>
														</div>
														<button
															onclick={() => unrankCandidate(candidate)}
															class="text-gray-400 hover:text-gray-600 focus:outline-none"
															disabled={isSubmitting}
															aria-label="Remove from rankings"
														>
															<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
																<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
															</svg>
														</button>
													</div>
												</div>
											{/each}
										{/if}
									</div>
								</div>

								<!-- Unranked Candidates (gray zone) -->
								<div>
									<h3 class="text-md font-medium text-gray-900 mb-3">Available Candidates</h3>
									<div 
										class="min-h-32 bg-gray-50 border-2 border-gray-200 border-dashed rounded-lg p-4"
										use:dndzone={{
											items: unrankedCandidates,
											flipDurationMs: 200,
											dragDisabled: isSubmitting
										}}
										onconsider={handleUnrankedDrop}
										onfinalize={handleUnrankedDrop}
									>
										{#if unrankedCandidates.length === 0}
											<p class="text-gray-500 text-center py-8">All candidates have been ranked</p>
										{:else}
											{#each unrankedCandidates as candidate (candidate.id)}
												<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-3 mb-2 cursor-move hover:shadow-md transition-shadow">
													<div class="flex items-center justify-between">
														<div>
															<h4 class="text-sm font-medium text-gray-900">{candidate.name}</h4>
															{#if candidate.description}
																<p class="text-xs text-gray-500">{candidate.description}</p>
															{/if}
														</div>
														<button
															data-testid="rank-candidate-{candidate.id.replace('_drag', '')}"
															onclick={() => rankCandidate(candidate)}
															class="text-indigo-600 hover:text-indigo-800 focus:outline-none text-sm font-medium"
															disabled={isSubmitting}
														>
															Rank
														</button>
													</div>
												</div>
											{/each}
										{/if}
									</div>
								</div>
							</div>

							<!-- Submit Section (matches vote page) -->
							<div class="mt-8 border-t border-gray-200 pt-6 flex justify-between items-center">
								<div class="text-sm text-gray-600">
									{#if rankedCandidates.length > 0}
										<p>You have ranked {rankedCandidates.length} candidate{rankedCandidates.length === 1 ? '' : 's'}</p>
									{:else}
										<p>Please rank at least one candidate</p>
									{/if}
								</div>
								<div class="flex space-x-3">
									<button
										onclick={cancelVoting}
										class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
									>
										Cancel
									</button>
									<button
										data-testid="submit-ballot-btn"
										onclick={submitVote}
										disabled={rankedCandidates.length === 0 || isSubmitting}
										class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-300 disabled:cursor-not-allowed"
									>
										{#if isSubmitting}
											<svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
												<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
												<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
											</svg>
											Submitting...
										{:else}
											Submit Ballot
										{/if}
									</button>
								</div>
							</div>
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

<style>
	/* DnD styles (matches vote page) */
	:global([data-dnd-zone] .dnd-drop-zone) {
		opacity: 0.8;
	}
	:global([data-dnd-zone] .dnd-drag-ghost) {
		opacity: 0.5;
		transform: rotate(5deg);
	}
</style>