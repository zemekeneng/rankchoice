<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { apiClient } from '$lib/api/client.js';
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

	// Color palette for candidates (consistent with RCV component)
	let candidateColors = $derived.by(() => {
		const colors = [
			'#3b82f6', // blue
			'#ef4444', // red  
			'#10b981', // green
			'#f59e0b', // amber
			'#8b5cf6', // violet
			'#06b6d4', // cyan
			'#f97316', // orange
			'#84cc16', // lime
			'#ec4899', // pink
			'#6b7280'  // gray
		];
		
		const colorMap: Record<string, string> = {};
		candidates.forEach((candidate, index) => {
			colorMap[candidate.id] = colors[index % colors.length];
		});
		return colorMap;
	});

	// Load poll data
	async function loadResults() {
		if (!pollId) return;

		try {
			isLoading = true;
			error = null;

			// Load poll details
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
	onMount(() => {
		loadResults();
	});

	// Auto-print when loaded (with delay for content rendering)
	$effect(() => {
		if (!isLoading && poll && results && rounds.length > 0) {
			setTimeout(() => {
				window.print();
			}, 1000);
		}
	});

	// Format percentage
	function formatPercentage(value: number): string {
		return `${value.toFixed(1)}%`;
	}

	// Format date
	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString();
	}

	// Get candidate name by ID
	function getCandidateName(candidateId: string): string {
		return candidates.find(c => c.id === candidateId)?.name || 'Unknown';
	}

	// Calculate majority threshold
	let majorityThreshold = $derived(() => {
		if (!results) return 0;
		return Math.floor(results.totalVotes / 2) + 1;
	});
</script>

<svelte:head>
	<title>Print Results - {poll?.title || 'Poll Results'}</title>
	<style>
		@media print {
			body { margin: 0; }
			.no-print { display: none !important; }
			.page-break { page-break-before: always; }
			.avoid-break { page-break-inside: avoid; }
		}
	</style>
</svelte:head>

{#if isLoading}
	<div class="flex items-center justify-center min-h-screen">
		<div class="text-center">
			<div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600 mx-auto mb-4"></div>
			<p class="text-gray-600">Loading results for printing...</p>
		</div>
	</div>
{:else if error}
	<div class="max-w-4xl mx-auto py-8 px-4">
		<div class="bg-red-50 border border-red-200 rounded-md p-4">
			<h2 class="text-red-800 font-semibold">Error Loading Results</h2>
			<p class="text-red-600">{error}</p>
		</div>
	</div>
{:else if poll && results}
	<!-- Header Section -->
	<div class="max-w-4xl mx-auto p-6 avoid-break">
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-gray-900 mb-2">{poll.title}</h1>
			{#if poll.description}
				<p class="text-lg text-gray-600 mb-4">{poll.description}</p>
			{/if}
			<div class="text-sm text-gray-500 space-y-1">
				<p><strong>Poll Type:</strong> {poll.pollType === 'single_winner' ? 'Single Winner' : `${poll.numWinners} Winners`} Ranked Choice Voting</p>
				<p><strong>Total Votes:</strong> {results.totalVotes}</p>
				<p><strong>Majority Threshold:</strong> {majorityThreshold} votes (50% + 1)</p>
				<p><strong>Results Generated:</strong> {formatDate(new Date().toISOString())}</p>
			</div>
		</div>

		<!-- Winner Announcement -->
		{#if results.winner}
			<div class="bg-green-50 border-2 border-green-200 rounded-lg p-6 mb-8 text-center avoid-break">
				<h2 class="text-2xl font-bold text-green-800 mb-2">🏆 Winner</h2>
				<p class="text-xl font-semibold text-green-700">{results.winner.name}</p>
				<p class="text-green-600">{results.winner.finalVotes} votes ({formatPercentage(results.winner.percentage)})</p>
			</div>
		{/if}

		<!-- Final Rankings -->
		<div class="mb-8 avoid-break">
			<h2 class="text-xl font-bold text-gray-900 mb-4">Final Rankings</h2>
			<div class="bg-white border border-gray-200 rounded-lg overflow-hidden">
				<table class="w-full">
					<thead class="bg-gray-50">
						<tr>
							<th class="px-4 py-3 text-left text-sm font-medium text-gray-900">Position</th>
							<th class="px-4 py-3 text-left text-sm font-medium text-gray-900">Candidate</th>
							<th class="px-4 py-3 text-right text-sm font-medium text-gray-900">Final Votes</th>
							<th class="px-4 py-3 text-right text-sm font-medium text-gray-900">Percentage</th>
							<th class="px-4 py-3 text-center text-sm font-medium text-gray-900">Eliminated</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-gray-200">
						{#each results.finalRankings as ranking}
							<tr class:bg-green-50={ranking.position === 1}>
								<td class="px-4 py-3 text-sm font-medium text-gray-900">#{ranking.position}</td>
								<td class="px-4 py-3 text-sm text-gray-900">{ranking.name}</td>
								<td class="px-4 py-3 text-sm text-gray-900 text-right">{ranking.votes}</td>
								<td class="px-4 py-3 text-sm text-gray-900 text-right">{formatPercentage(ranking.percentage)}</td>
								<td class="px-4 py-3 text-sm text-gray-500 text-center">
									{ranking.eliminatedRound ? `Round ${ranking.eliminatedRound}` : '—'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	</div>

	<!-- Round-by-Round Results -->
	<div class="page-break"></div>
	<div class="max-w-4xl mx-auto p-6">
		<h1 class="text-2xl font-bold text-gray-900 mb-6 text-center">Round-by-Round Results</h1>
		
		{#each rounds as round, index}
			<div class="mb-8 avoid-break" class:page-break={index > 0 && index % 2 === 0}>
				<div class="bg-white border-2 border-gray-200 rounded-lg overflow-hidden">
					<!-- Round Header -->
					<div class="bg-blue-50 px-6 py-4 border-b border-gray-200">
						<div class="flex justify-between items-center">
							<h3 class="text-lg font-semibold text-gray-900">Round {round.roundNumber}</h3>
							<div class="text-sm text-gray-600">
								<span class="font-medium">Active Votes:</span> {round.total_votes}
								{#if round.exhausted_ballots > 0}
									<span class="ml-4 font-medium">Exhausted:</span> {round.exhausted_ballots}
								{/if}
							</div>
						</div>
					</div>

					<!-- Vote Counts Table -->
					<div class="p-6">
						<table class="w-full mb-4">
							<thead>
								<tr class="border-b border-gray-200">
									<th class="text-left py-2 text-sm font-medium text-gray-900">Candidate</th>
									<th class="text-right py-2 text-sm font-medium text-gray-900">Votes</th>
									<th class="text-right py-2 text-sm font-medium text-gray-900">Percentage</th>
									<th class="text-center py-2 text-sm font-medium text-gray-900">Status</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-gray-100">
								{#each Object.entries(round.vote_counts) as [candidateId, voteData]}
									{@const candidate = candidates.find(c => c.id === candidateId)}
									{@const isEliminated = round.eliminated && (
										typeof round.eliminated === 'string' ? round.eliminated === candidateId : round.eliminated.candidate_id === candidateId
									)}
									{@const isWinner = round.winner && (
										typeof round.winner === 'string' ? round.winner === candidateId : round.winner.candidate_id === candidateId
									)}
									<tr class:bg-red-50={isEliminated} class:bg-green-50={isWinner}>
										<td class="py-3 text-sm font-medium text-gray-900 flex items-center">
											<div 
												class="w-3 h-3 rounded-full mr-3"
												style="background-color: {candidateColors[candidateId] || '#6b7280'}"
											></div>
											{candidate?.name || getCandidateName(candidateId)}
										</td>
										<td class="py-3 text-sm text-gray-900 text-right font-mono">{voteData.votes}</td>
										<td class="py-3 text-sm text-gray-900 text-right">{formatPercentage(voteData.percentage)}</td>
										<td class="py-3 text-sm text-center">
											{#if isWinner}
												<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
													🏆 Winner
												</span>
											{:else if isEliminated}
												<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
													❌ Eliminated
												</span>
											{:else}
												<span class="text-gray-500">Active</span>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>

						<!-- Round Summary -->
						<div class="bg-gray-50 rounded-lg p-4 text-sm">
							<div class="grid grid-cols-2 gap-4">
								<div>
									<span class="font-medium text-gray-700">Majority Threshold:</span>
									<span class="text-gray-900">{round.majority_threshold} votes</span>
								</div>
								{#if round.eliminated}
									<div>
										<span class="font-medium text-gray-700">Eliminated:</span>
										<span class="text-red-700">
											{typeof round.eliminated === 'string' 
												? getCandidateName(round.eliminated)
												: round.eliminated.name}
										</span>
									</div>
								{/if}
								{#if round.tiebreak_reason}
									<div class="col-span-2">
										<span class="font-medium text-gray-700">Tiebreaker Used:</span>
										<span class="text-orange-700">
											{#if round.tiebreak_reason === 'FirstChoiceVotes'}
												Fewest first-choice votes
											{:else if round.tiebreak_reason === 'PriorRoundPerformance'}
												Prior round performance
											{:else if round.tiebreak_reason === 'MostVotesToDistribute'}
												Most votes to redistribute
											{:else if round.tiebreak_reason === 'Random'}
												Random selection
											{:else}
												{round.tiebreak_reason}
											{/if}
										</span>
									</div>
								{/if}
							</div>
						</div>
					</div>
				</div>
			</div>
		{/each}
	</div>

	<!-- Footer -->
	<div class="max-w-4xl mx-auto p-6 text-center text-sm text-gray-500 border-t">
		<p>Generated by RankChoice.app - Secure, transparent, democratic voting</p>
		<p>Learn more about ranked choice voting at <strong>rankchoice.app</strong></p>
	</div>
{:else}
	<div class="max-w-4xl mx-auto py-8 px-4">
		<div class="text-center">
			<p class="text-gray-600">No results available for this poll.</p>
		</div>
	</div>
{/if}

<style>
	/* Print-specific styles */
	@media print {
		body {
			font-size: 12px;
			line-height: 1.4;
		}
		
		.avoid-break {
			page-break-inside: avoid;
		}
		
		.page-break {
			page-break-before: always;
		}
		
		table {
			border-collapse: collapse;
		}
		
		th, td {
			border: 1px solid #e5e7eb;
		}
	}

	/* Screen styles for preview */
	@media screen {
		body {
			background-color: #f3f4f6;
		}
	}
</style>