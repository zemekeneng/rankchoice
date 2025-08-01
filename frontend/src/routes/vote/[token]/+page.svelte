<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { dndzone } from 'svelte-dnd-action';
	import { apiClient } from '$lib/api/client.js';
	import type { Poll, Candidate } from '$lib/types.js';

	// Get token from URL parameter
	let token = $derived($page.params.token);

	// State management
	let poll = $state<Poll | null>(null);
	let candidates = $state<Candidate[]>([]);
	let rankedCandidates = $state<Candidate[]>([]);
	let unrankedCandidates = $state<Candidate[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let submitting = $state(false);
	let hasVoted = $state(false);
	let submitted = $state(false);
	let receipt = $state<any>(null);

	// Load ballot data
	onMount(async () => {
		if (!token) {
			error = 'Invalid ballot token';
			loading = false;
			return;
		}

		try {
			const data = await apiClient.getBallot(token);
			poll = data.poll;
			candidates = data.poll.candidates || [];
			hasVoted = data.voter.has_voted;

			if (hasVoted) {
				// User has already voted, show receipt
				try {
					receipt = await apiClient.getVotingReceipt(token!);
					submitted = true;
				} catch (e) {
					// If we can't get receipt, still show they've voted
				}
			} else {
				// Initialize candidates for ranking
				unrankedCandidates = [...candidates].map((c, index) => ({
					...c,
					id: c.id + '_drag', // Add unique ID for DnD
					rank: undefined
				}));
				rankedCandidates = [];
			}
		} catch (e: any) {
			error = e.message || 'Failed to load ballot';
		} finally {
			loading = false;
		}
	});

	// Drag and drop handlers
	function handleRankedDrop(event: CustomEvent) {
		rankedCandidates = event.detail.items;
		updateRanks();
	}

	function handleUnrankedDrop(event: CustomEvent) {
		unrankedCandidates = event.detail.items;
	}

	function updateRanks() {
		rankedCandidates = rankedCandidates.map((candidate, index) => ({
			...candidate,
			rank: index + 1
		}));
	}

	// Move candidate to ranked list
	function rankCandidate(candidate: Candidate) {
		unrankedCandidates = unrankedCandidates.filter(c => c.id !== candidate.id);
		rankedCandidates = [...rankedCandidates, { ...candidate, rank: rankedCandidates.length + 1 }];
	}

	// Move candidate back to unranked list
	function unrankCandidate(candidate: Candidate) {
		rankedCandidates = rankedCandidates.filter(c => c.id !== candidate.id);
		unrankedCandidates = [...unrankedCandidates, { ...candidate, rank: undefined }];
		updateRanks();
	}

	// Submit ballot
	async function submitBallot() {
		if (!token) {
			error = 'Invalid ballot token';
			return;
		}

		if (rankedCandidates.length === 0) {
			error = 'Please rank at least one candidate';
			return;
		}

		submitting = true;
		error = null;

		try {
			const rankings = rankedCandidates.map(candidate => ({
				candidate_id: candidate.id.replace('_drag', ''), // Remove drag suffix
				rank: candidate.rank!
			}));

			const result = await apiClient.submitBallot(token, rankings);
			receipt = result.receipt;
			submitted = true;
		} catch (e: any) {
			error = e.message || 'Failed to submit ballot';
		} finally {
			submitting = false;
		}
	}

	// Helper to check if ballot is valid
	let canSubmit = $derived(rankedCandidates.length > 0 && !submitting);
</script>

<svelte:head>
	<title>Vote - {poll?.title || 'Loading...'}</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 py-8">
	<div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
		{#if loading}
			<div class="text-center py-12" data-testid="voting-loading">
				<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
				<p class="mt-4 text-gray-600">Loading ballot...</p>
			</div>
		{:else if error}
			<div class="bg-red-50 border border-red-200 rounded-md p-4" data-testid="voting-error">
				<div class="flex">
					<div class="flex-shrink-0">
						<svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
						</svg>
					</div>
					<div class="ml-3">
						<h3 class="text-sm font-medium text-red-800">Error loading ballot</h3>
						<p class="mt-2 text-sm text-red-700">{error}</p>
					</div>
				</div>
			</div>
		{:else if submitted}
			<!-- Receipt -->
			<div class="bg-white shadow rounded-lg" data-testid="voting-receipt">
				<div class="px-6 py-4 border-b border-gray-200">
					<h1 class="text-2xl font-bold text-green-600" data-testid="vote-success-heading">Vote Submitted Successfully!</h1>
					<p class="mt-2 text-gray-600" data-testid="vote-success-message">Thank you for participating in this ranked-choice voting poll.</p>
				</div>
				
				{#if receipt}
					<div class="p-6">
						<h2 class="text-lg font-medium text-gray-900 mb-4" data-testid="receipt-heading">Your Voting Receipt</h2>
						<div class="bg-gray-50 rounded-lg p-4" data-testid="receipt-content">
							<dl class="space-y-2">
								<div>
									<dt class="text-sm font-medium text-gray-500">Poll:</dt>
									<dd class="text-sm text-gray-900" data-testid="receipt-poll-title">{poll?.title}</dd>
								</div>
								<div>
									<dt class="text-sm font-medium text-gray-500">Verification Code:</dt>
									<dd class="text-sm font-mono text-gray-900" data-testid="receipt-verification-code">{receipt.verification_code}</dd>
								</div>
								<div>
									<dt class="text-sm font-medium text-gray-500">Submitted At:</dt>
									<dd class="text-sm text-gray-900" data-testid="receipt-submitted-at">{new Date(receipt.submitted_at).toLocaleString()}</dd>
								</div>
							</dl>
						</div>
						<p class="mt-4 text-sm text-gray-500" data-testid="receipt-instructions">
							Keep this verification code for your records. You can use it to verify your vote was counted.
						</p>
					</div>
				{/if}
				
				<div class="px-6 py-4 border-t border-gray-200">
					<div class="text-center">
						<button
							data-testid="return-home-btn"
							onclick={() => goto('/')}
							class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
						>
							Return to Home
						</button>
					</div>
				</div>
			</div>
		{:else if poll}
			<!-- Voting interface -->
			<div class="bg-white shadow rounded-lg" data-testid="voting-interface">
				<div class="px-6 py-4 border-b border-gray-200" data-testid="poll-header">
					<h1 class="text-2xl font-bold text-gray-900" data-testid="poll-title">{poll.title}</h1>
					{#if poll.description}
						<p class="mt-2 text-gray-600" data-testid="poll-description">{poll.description}</p>
					{/if}
					<div class="mt-3 flex flex-wrap gap-2 text-sm text-gray-500" data-testid="poll-info">
						<span class="inline-flex items-center px-2.5 py-0.5 rounded-full bg-indigo-100 text-indigo-800" data-testid="poll-type-badge">
							{poll.pollType === 'single_winner' ? 'Single Winner' : `${poll.numWinners} Winners`}
						</span>
						<span data-testid="voting-method">Ranked Choice Voting</span>
					</div>
				</div>

				<div class="p-6">
					<div class="mb-6">
						<h2 class="text-lg font-medium text-gray-900 mb-2" data-testid="instructions-heading">Instructions</h2>
						<div class="bg-blue-50 border border-blue-200 rounded-md p-4" data-testid="voting-instructions">
							<p class="text-sm text-blue-800">
								Drag and drop candidates to rank them in order of your preference. Your #1 choice should be at the top.
								You can rank as many or as few candidates as you like.
							</p>
						</div>
					</div>

					<!-- Mobile-friendly layout -->
					<div class="lg:grid lg:grid-cols-2 lg:gap-6">
						<!-- Ranked Candidates (Left side on desktop) -->
						<div class="mb-6 lg:mb-0">
							<h3 class="text-md font-medium text-gray-900 mb-3" data-testid="ranked-section-heading">Your Rankings</h3>
							<div 
								class="min-h-32 bg-green-50 border-2 border-green-200 border-dashed rounded-lg p-4"
								data-testid="ranked-candidates-zone"
								use:dndzone={{
									items: rankedCandidates,
									flipDurationMs: 200,
									dragDisabled: submitting
								}}
								onconsider={handleRankedDrop}
								onfinalize={handleRankedDrop}
							>
								{#if rankedCandidates.length === 0}
									<p class="text-green-600 text-center py-8" data-testid="ranked-empty-state">Drop candidates here to rank them</p>
								{:else}
									{#each rankedCandidates as candidate (candidate.id)}
										<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-3 mb-2 cursor-move hover:shadow-md transition-shadow" data-testid="ranked-candidate-{candidate.id.replace('_drag', '')}">
											<div class="flex items-center justify-between">
												<div class="flex items-center space-x-3">
													<div class="flex-shrink-0 w-8 h-8 bg-green-100 rounded-full flex items-center justify-center" data-testid="candidate-rank-badge-{candidate.id.replace('_drag', '')}">
														<span class="text-sm font-medium text-green-800">#{candidate.rank}</span>
													</div>
													<div>
														<h4 class="text-sm font-medium text-gray-900" data-testid="ranked-candidate-name-{candidate.id.replace('_drag', '')}">{candidate.name}</h4>
														{#if candidate.description}
															<p class="text-xs text-gray-500" data-testid="ranked-candidate-description-{candidate.id.replace('_drag', '')}">{candidate.description}</p>
														{/if}
													</div>
												</div>
												<button
													data-testid="unrank-candidate-btn-{candidate.id.replace('_drag', '')}"
													onclick={() => unrankCandidate(candidate)}
													class="text-gray-400 hover:text-gray-600 focus:outline-none"
													disabled={submitting}
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

						<!-- Unranked Candidates (Right side on desktop) -->
						<div>
							<h3 class="text-md font-medium text-gray-900 mb-3" data-testid="unranked-section-heading">Available Candidates</h3>
							<div 
								class="min-h-32 bg-gray-50 border-2 border-gray-200 border-dashed rounded-lg p-4"
								data-testid="unranked-candidates-zone"
								use:dndzone={{
									items: unrankedCandidates,
									flipDurationMs: 200,
									dragDisabled: submitting
								}}
								onconsider={handleUnrankedDrop}
								onfinalize={handleUnrankedDrop}
							>
								{#if unrankedCandidates.length === 0}
									<p class="text-gray-500 text-center py-8" data-testid="unranked-empty-state">All candidates have been ranked</p>
								{:else}
									{#each unrankedCandidates as candidate (candidate.id)}
										<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-3 mb-2 cursor-move hover:shadow-md transition-shadow" data-testid="unranked-candidate-{candidate.id.replace('_drag', '')}">
											<div class="flex items-center justify-between">
												<div>
													<h4 class="text-sm font-medium text-gray-900" data-testid="unranked-candidate-name-{candidate.id.replace('_drag', '')}">{candidate.name}</h4>
													{#if candidate.description}
														<p class="text-xs text-gray-500" data-testid="unranked-candidate-description-{candidate.id.replace('_drag', '')}">{candidate.description}</p>
													{/if}
												</div>
												<button
													data-testid="rank-candidate-btn-{candidate.id.replace('_drag', '')}"
													onclick={() => rankCandidate(candidate)}
													class="text-indigo-600 hover:text-indigo-800 focus:outline-none text-sm font-medium"
													disabled={submitting}
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

					<!-- Submit Section -->
					<div class="mt-8 border-t border-gray-200 pt-6" data-testid="submit-section">
						{#if error}
							<div class="mb-4 bg-red-50 border border-red-200 rounded-md p-3" data-testid="submit-error">
								<p class="text-sm text-red-600">{error}</p>
							</div>
						{/if}

						<div class="flex justify-between items-center">
							<div class="text-sm text-gray-600" data-testid="ranking-summary">
								{#if rankedCandidates.length > 0}
									<p>You have ranked {rankedCandidates.length} candidate{rankedCandidates.length === 1 ? '' : 's'}</p>
								{:else}
									<p>Please rank at least one candidate to submit your ballot</p>
								{/if}
							</div>
							
							<button
								data-testid="submit-ballot-btn"
								onclick={submitBallot}
								disabled={!canSubmit}
								class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-300 disabled:cursor-not-allowed"
							>
								{#if submitting}
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
		{/if}
	</div>
</div>

<style>
	/* Custom styles for drag and drop */
	:global([data-dnd-zone] .dnd-drop-zone) {
		opacity: 0.8;
	}
	
	:global([data-dnd-zone] .dnd-drag-ghost) {
		opacity: 0.5;
		transform: rotate(5deg);
	}
</style> 