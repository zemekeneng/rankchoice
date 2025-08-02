<script lang="ts">
	import type { RCVRound, Candidate } from '$lib/types.js';
	import { flip } from 'svelte/animate';

	interface Props {
		rounds: RCVRound[];
		candidates: Candidate[];
		totalVotes: number;
	}

	let { rounds, candidates, totalVotes }: Props = $props();

	// State management
	let currentRound = $state(0);
	let isPlaying = $state(false);
	let playbackSpeed = $state(1300); // ms between rounds (optimized for FLIP animation)
	let allowSorting = $state(true); // Controls when sorting should happen
	let frozenOrder = $state<Candidate[]>([]); // Preserves order during Stage 1

	// Derived state
	let maxRound = $derived(rounds.length - 1);
	let currentRoundData = $derived(rounds[currentRound] || null);
	let majorityThreshold = $derived(Math.floor(totalVotes / 2) + 1);
	
	// Sorted candidates by vote count (non-mutating) - uses peak votes for eliminated candidates
	// Only sorts when allowSorting is true, otherwise preserves frozen order
	let sortedCandidates = $derived.by(() => {
		if (allowSorting) {
			return [...candidates].sort((a, b) => {
				const aVotes = getSortingVoteCount(a.id);
				const bVotes = getSortingVoteCount(b.id);
				return bVotes - aVotes;
			});
		} else {
			// Use frozen order to prevent jumping during Stage 1
			const order = frozenOrder.length > 0 ? frozenOrder : [...candidates];
			return [...order];
		}
	});

	// Update frozen order when sorting is enabled (separate from derived)
	$effect(() => {
		if (allowSorting && sortedCandidates.length > 0) {
			frozenOrder = [...sortedCandidates];
		}
	});

	// Color palette for candidates
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

	// Get vote percentage for a candidate in current round
	function getVotePercentage(candidateId: string): number {
		if (!currentRoundData) return 0;
		const voteData = currentRoundData.vote_counts[candidateId];
		const votes = voteData ? voteData.votes : 0;
		return totalVotes > 0 ? (votes / totalVotes) * 100 : 0;
	}

	// Get vote count for a candidate in current round
	function getVoteCount(candidateId: string): number {
		if (!currentRoundData) return 0;
		const voteData = currentRoundData.vote_counts[candidateId];
		return voteData ? voteData.votes : 0;
	}

	// Check if candidate is eliminated in current round
	function isEliminated(candidateId: string): boolean {
		if (!currentRoundData || !currentRoundData.eliminated) return false;
		const eliminatedId = typeof currentRoundData.eliminated === 'string' 
			? currentRoundData.eliminated 
			: currentRoundData.eliminated.candidate_id;
		return eliminatedId === candidateId;
	}

	// Check if candidate won in current round
	function isWinner(candidateId: string): boolean {
		if (!currentRoundData || !currentRoundData.winner) return false;
		const winnerId = typeof currentRoundData.winner === 'string' 
			? currentRoundData.winner 
			: currentRoundData.winner.candidate_id;
		return winnerId === candidateId;
	}

	// Check if candidate is still active (not eliminated in previous rounds)
	function isActive(candidateId: string): boolean {
		for (let i = 0; i < currentRound; i++) {
			if (rounds[i] && rounds[i].eliminated) {
				const eliminated = rounds[i].eliminated!;
				const eliminatedId = typeof eliminated === 'string' 
					? eliminated 
					: eliminated.candidate_id;
				if (eliminatedId === candidateId) {
					return false;
				}
			}
		}
		return true;
	}

	// Get votes from previous round for comparison
	function getPreviousRoundVotes(candidateId: string): number {
		if (currentRound === 0) return 0;
		const prevRound = rounds[currentRound - 1];
		if (!prevRound) return 0;
		const voteData = prevRound.vote_counts[candidateId];
		return voteData ? voteData.votes : 0;
	}

	// Calculate transferred votes (difference from previous round)
	function getTransferredVotes(candidateId: string): number {
		// In the first round, there are no transferred votes
		if (currentRound === 0) {
			return 0;
		}
		
		// If candidate is not active, they can't receive transfers
		if (!isActive(candidateId)) {
			return 0;
		}
		
		const currentVotes = getVoteCount(candidateId);
		const previousVotes = getPreviousRoundVotes(candidateId);
		return Math.max(0, currentVotes - previousVotes);
	}

	// Get base votes (votes from previous round, or 0 if eliminated)
	function getBaseVotes(candidateId: string): number {
		// If this candidate is currently eliminated, they have no base votes in this round
		if (!isActive(candidateId)) return 0;
		
		// In the first round, all votes are base votes
		if (currentRound === 0) {
			return getVoteCount(candidateId);
		}
		
		// Otherwise, their base votes are what they had in the previous round
		return getPreviousRoundVotes(candidateId);
	}

	// Calculate percentage of base votes
	function getBaseVotePercentage(candidateId: string): number {
		const baseVotes = getBaseVotes(candidateId);
		return totalVotes > 0 ? (baseVotes / totalVotes) * 100 : 0;
	}

	// Calculate percentage of transferred votes
	function getTransferredVotePercentage(candidateId: string): number {
		const transferredVotes = getTransferredVotes(candidateId);
		return totalVotes > 0 ? (transferredVotes / totalVotes) * 100 : 0;
	}

	// Get vote distribution for eliminated candidates (where their votes went)
	function getVoteDistribution(eliminatedCandidateId: string): Record<string, number> {
		const distribution: Record<string, number> = {};
		
		if (currentRound === 0) return distribution;
		
		// Find the round where this candidate was eliminated
		let eliminationRound = -1;
		for (let i = 0; i <= currentRound; i++) { // Changed < to <= to include current round
			if (rounds[i] && rounds[i].eliminated) {
				const eliminated = rounds[i].eliminated!;
				const eliminatedId = typeof eliminated === 'string' 
					? eliminated 
					: eliminated.candidate_id;
				if (eliminatedId === eliminatedCandidateId) {
					eliminationRound = i;
					break;
				}
			}
		}
		
		if (eliminationRound === -1) return distribution;
		
		// Calculate how votes were distributed in the next round
		const beforeElimination = rounds[eliminationRound];
		const afterElimination = rounds[eliminationRound + 1];
		
		if (!beforeElimination || !afterElimination) return distribution;
		
		// Calculate the increase for each remaining candidate
		candidates.forEach(candidate => {
			if (candidate.id === eliminatedCandidateId) return;
			
			const beforeVotes = beforeElimination.vote_counts[candidate.id]?.votes || 0;
			const afterVotes = afterElimination.vote_counts[candidate.id]?.votes || 0;
			const increase = afterVotes - beforeVotes;
			
			if (increase > 0) {
				distribution[candidate.id] = increase;
			}
		});
		return distribution;
	}

	// Get the vote count for sorting (current votes or votes when eliminated)
	function getSortingVoteCount(candidateId: string): number {
		// First check if candidate is eliminated in the current round
		if (currentRoundData && currentRoundData.eliminated) {
			const eliminated = currentRoundData.eliminated!;
			const eliminatedId = typeof eliminated === 'string' 
				? eliminated 
				: eliminated.candidate_id;
			if (eliminatedId === candidateId) {
				// For currently eliminated candidate, use their vote count from this round
				const voteData = currentRoundData.vote_counts[candidateId];
				return voteData ? voteData.votes : 0;
			}
		}
		
		// Check if candidate was eliminated in a previous round
		for (let i = 0; i < currentRound; i++) {
			if (rounds[i] && rounds[i].eliminated) {
				const eliminated = rounds[i].eliminated!;
				const eliminatedId = typeof eliminated === 'string' 
					? eliminated 
					: eliminated.candidate_id;
				if (eliminatedId === candidateId) {
					// Return the vote count from the round they were eliminated
					const voteData = rounds[i].vote_counts[candidateId];
					return voteData ? voteData.votes : 0;
				}
			}
		}
		
		// If candidate is still active, use current vote count
		return getVoteCount(candidateId);
	}

	// Create darker shade of color for transferred votes
	function getDarkerColor(color: string, factor: number = 0.6): string {
		// Handle hex colors
		if (color.startsWith('#')) {
			const hex = color.replace('#', '');
			const r = parseInt(hex.substr(0, 2), 16);
			const g = parseInt(hex.substr(2, 2), 16);
			const b = parseInt(hex.substr(4, 2), 16);
			
			const darkerR = Math.floor(r * factor);
			const darkerG = Math.floor(g * factor);
			const darkerB = Math.floor(b * factor);
			
			return `rgb(${darkerR}, ${darkerG}, ${darkerB})`;
		}
		
		// Handle rgb colors or fallback
		return color;
	}

	// Two-stage animation: first update votes, then update order
	async function animateRoundChange(newRound: number) {
		// Only animate if we're actually changing rounds
		if (newRound === currentRound) return;
		
		// Stage 1: Disable sorting, update round (shows vote redistribution)
		allowSorting = false;
		currentRound = newRound;
		
		// Wait for vote redistribution to be visible
		await new Promise(resolve => setTimeout(resolve, 500));
		
		// Stage 2: Enable sorting (animates reordering)
		allowSorting = true;
	}

	// Navigation functions
	async function nextRound() {
		if (currentRound < maxRound) {
			await animateRoundChange(currentRound + 1);
		}
	}

	async function prevRound() {
		if (currentRound > 0) {
			await animateRoundChange(currentRound - 1);
		}
	}

	async function goToRound(round: number) {
		const newRound = Math.max(0, Math.min(round, maxRound));
		if (newRound !== currentRound) {
			await animateRoundChange(newRound);
		}
	}

	// Auto-play functionality
	let playInterval: number | null = null;

	function togglePlay() {
		isPlaying = !isPlaying;
		
		if (isPlaying) {
			playInterval = setInterval(async () => {
				if (currentRound < maxRound) {
					await nextRound();
				} else {
					isPlaying = false;
					if (playInterval) clearInterval(playInterval);
				}
			}, playbackSpeed);
		} else {
			if (playInterval) {
				clearInterval(playInterval);
				playInterval = null;
			}
		}
	}

	async function reset() {
		isPlaying = false;
		if (playInterval) {
			clearInterval(playInterval);
			playInterval = null;
		}
		await animateRoundChange(0);
	}

	// Initialize sorting on component mount
	$effect(() => {
		// Ensure sorting is enabled on initial load and initialize frozen order
		allowSorting = true;
		if (frozenOrder.length === 0) {
			frozenOrder = [...candidates];
		}
	});

	// Cleanup on component destroy
	$effect(() => {
		return () => {
			if (playInterval) {
				clearInterval(playInterval);
				playInterval = null;
			}
		};
	});
</script>

<div class="bg-white shadow rounded-lg overflow-hidden">
	<!-- Header -->
	<div class="px-6 py-4 border-b border-gray-200">
		<div class="flex items-center justify-between">
			<div>
				<h3 class="text-lg font-medium text-gray-900">Interactive RCV Visualization</h3>
				<p class="text-sm text-gray-500">
					Round {currentRound + 1} of {rounds.length} 
					{#if currentRoundData?.eliminated}
						• {typeof currentRoundData.eliminated === 'string' 
							? candidates.find(c => c.id === currentRoundData.eliminated)?.name 
							: currentRoundData.eliminated.name} eliminated
					{:else if currentRoundData?.winner}
						• {typeof currentRoundData.winner === 'string' 
							? candidates.find(c => c.id === currentRoundData.winner)?.name 
							: currentRoundData.winner.name} wins!
					{/if}
				</p>
			</div>
			<div class="text-sm text-gray-500">
				Majority needed: {majorityThreshold} votes
			</div>
		</div>
	</div>

	<!-- Controls -->
	<div class="px-6 py-4 bg-gray-50 border-b border-gray-200">
		<div class="flex items-center justify-between">
			<div class="flex items-center space-x-2">
				<button
					onclick={reset}
					disabled={currentRound === 0}
					class="px-3 py-1 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					⏮ Reset
				</button>
				<button
					onclick={prevRound}
					disabled={currentRound === 0}
					class="px-3 py-1 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					⏮ Prev
				</button>
				<button
					onclick={togglePlay}
					disabled={currentRound === maxRound && !isPlaying}
					class="px-4 py-1 text-sm bg-indigo-600 text-white rounded hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					{isPlaying ? '⏸ Pause' : '▶ Play'}
				</button>
				<button
					onclick={nextRound}
					disabled={currentRound === maxRound}
					class="px-3 py-1 text-sm bg-gray-200 text-gray-700 rounded hover:bg-gray-300 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					Next ⏭
				</button>
			</div>

			<!-- Round selector -->
			<div class="flex items-center space-x-2">
				<span class="text-sm text-gray-500">Round:</span>
				{#each rounds as round, index}
					<button
						onclick={() => goToRound(index)}
						class="w-8 h-8 text-xs rounded-full border-2 transition-all duration-200"
						class:bg-indigo-600={index === currentRound}
						class:text-white={index === currentRound}
						class:border-indigo-600={index === currentRound}
						class:bg-white={index !== currentRound}
						class:text-gray-600={index !== currentRound}
						class:border-gray-300={index !== currentRound}
						class:hover:border-indigo-400={index !== currentRound}
					>
						{index + 1}
					</button>
				{/each}
			</div>
		</div>

		<!-- Progress bar -->
		<div class="mt-4">
			<div class="flex justify-between text-sm text-gray-500 mb-1">
				<span>Round Progress</span>
				<span>{Math.round(((currentRound + 1) / rounds.length) * 100)}%</span>
			</div>
			<div class="w-full bg-gray-200 rounded-full h-2">
				<div 
					class="bg-indigo-600 h-2 rounded-full transition-all duration-300 ease-out"
					style="width: {((currentRound + 1) / rounds.length) * 100}%"
				></div>
			</div>
		</div>
	</div>

			<!-- Candidate Vote Visualization -->
		<div class="px-6 py-6 relative">
			<!-- 50% majority line spanning all candidates -->
			<div 
				class="absolute top-0 bottom-0 w-0.5 bg-yellow-500 z-40 pointer-events-none"
				style="left: 50%"
			>
				<div class="absolute -top-2 -left-12 text-xs text-yellow-600 font-medium whitespace-nowrap">
					Over 50% to win
				</div>
			</div>
			
			<div class="candidate-container">
			{#each sortedCandidates as candidate (candidate.id)}
				{@const votes = getVoteCount(candidate.id)}
				{@const percentage = getVotePercentage(candidate.id)}
				{@const active = isActive(candidate.id)}
				{@const eliminated = isEliminated(candidate.id)}
				{@const winner = isWinner(candidate.id)}
				{@const color = candidateColors[candidate.id]}
				{@const baseVotes = getBaseVotes(candidate.id)}
				{@const transferredVotes = getTransferredVotes(candidate.id)}
				{@const basePercentage = getBaseVotePercentage(candidate.id)}
				{@const transferredPercentage = getTransferredVotePercentage(candidate.id)}
				{@const darkerColor = getDarkerColor(color)}
				{@const shouldStripe = currentRound > 0 && votes === 0}
				{@const previouslyEliminated = shouldStripe && !eliminated}
				
				<div 
					class="relative p-4 rounded-lg border-2 candidate-card mb-4"
					class:bg-green-50={winner}
					class:border-green-300={winner}
					class:border-red-300={previouslyEliminated}
					class:bg-gray-50={previouslyEliminated}
					class:border-gray-300={previouslyEliminated}
					class:bg-white={!previouslyEliminated && !winner}
					class:border-gray-200={!previouslyEliminated && !winner}
					class:opacity-60={!active && !eliminated}
					animate:flip={{duration: 600}}
				>
					<!-- Candidate header -->
					<div class="flex items-center justify-between mb-3">
						<div class="flex items-center space-x-3">
							<div 
								class="w-4 h-4 rounded-full"
								style="background-color: {color}"
							></div>
							<div>
								<h4 class="font-medium text-gray-900">{candidate.name}</h4>
								{#if previouslyEliminated}
									<span class="text-xs text-red-600 font-medium">Eliminated</span>
								{:else if winner}
									<span class="text-xs text-green-600 font-medium">🏆 Winner!</span>
								{/if}
							</div>
						</div>
						
						<div class="text-right">
							<div class="text-lg font-semibold text-gray-900">{votes} votes</div>
							<div class="text-sm text-gray-500">{percentage.toFixed(1)}%</div>
						</div>
					</div>

					<!-- Vote progress bar -->
					<div class="relative">
						<div class="w-full bg-gray-200 h-6 overflow-hidden" style="border-radius: 0.375rem">
							{#if !previouslyEliminated}
								<!-- Base votes (from previous rounds) -->
								{#if basePercentage > 0}
									<div 
										class="h-6 transition-all duration-700 ease-out absolute left-0 top-0 z-10"
										style="width: {basePercentage}%; background-color: {color}; border-radius: {transferredPercentage > 0 ? '0.375rem 0 0 0.375rem' : '0.375rem'}"
									></div>
								{/if}
								
								<!-- Transferred votes (darker shade) -->
								{#if transferredPercentage > 0}
									<div 
										class="h-6 transition-all duration-700 ease-out absolute top-0 z-20"
										style="left: {basePercentage}%; width: {transferredPercentage}%; background-color: {darkerColor}; border-radius: 0 0.375rem 0.375rem 0"
									></div>
								{/if}
								
								<!-- Remove majority threshold indicator -->
							{:else}
								<!-- Previously eliminated candidate - show only distribution bar -->
								{@const showWidth = previouslyEliminated ? (currentRound === 0 ? percentage : getPreviousRoundVotes(candidate.id) / totalVotes * 100) : percentage}
								{@const distribution = getVoteDistribution(candidate.id)}
								
								{#if Object.keys(distribution).length > 0}
									<!-- Full-thickness distribution bar showing where votes went -->
									{@const totalDistributed = Object.values(distribution).reduce((sum, votes) => sum + votes, 0)}
									<div class="absolute top-0 left-0 h-6 z-20 flex" style="width: {showWidth}%; border-radius: 0.375rem; overflow: hidden">
										{#each Object.entries(distribution) as [receivingCandidateId, votesReceived], index}
											{@const distributionPercentage = (votesReceived / totalDistributed) * 100}
											{@const receivingColor = candidateColors[receivingCandidateId]}
											<div 
												class="h-6 transition-all duration-700 ease-out first:rounded-l-md last:rounded-r-md"
												style="width: {distributionPercentage}%; background-color: {receivingColor}"
												title="{votesReceived} votes to {candidates.find(c => c.id === receivingCandidateId)?.name}"
											></div>
										{/each}
									</div>
								{:else}
									<!-- If no distribution data, show the original colored bar -->
									<div 
										class="h-6 transition-all duration-700 ease-out absolute left-0 top-0 z-10"
										style="width: {showWidth}%; background-color: {color}; border-radius: 0.375rem"
									></div>
								{/if}
							{/if}
						</div>

					</div>


				</div>
			{/each}
		</div>

		<!-- Round summary -->
		{#if currentRoundData}
			<div class="mt-6 p-4 bg-gray-50 rounded-lg">
				<h4 class="font-medium text-gray-900 mb-2">Round {currentRound + 1} Summary</h4>
				<div class="text-sm text-gray-600 space-y-1">
					<div>Total votes counted: {Object.values(currentRoundData.vote_counts).reduce((sum, voteData) => sum + voteData.votes, 0)}</div>
					<div>Exhausted ballots: {currentRoundData.exhausted_ballots}</div>
					{#if currentRoundData.eliminated}
						<div class="text-red-600">
							{typeof currentRoundData.eliminated === 'string' 
								? candidates.find(c => c.id === currentRoundData.eliminated)?.name 
								: currentRoundData.eliminated.name} eliminated (lowest votes)
						</div>
						{#if currentRoundData.tiebreak_reason}
							<div class="text-orange-600 text-xs mt-1">
								<span class="font-medium">Tiebreaker used:</span>
								{#if currentRoundData.tiebreak_reason === 'FirstChoiceVotes'}
									Fewest first-choice votes
								{:else if currentRoundData.tiebreak_reason === 'PriorRoundPerformance'}
									Prior round performance
								{:else if currentRoundData.tiebreak_reason === 'MostVotesToDistribute'}
									Most votes to redistribute
								{:else if currentRoundData.tiebreak_reason === 'Random'}
									Random selection
								{:else}
									{currentRoundData.tiebreak_reason}
								{/if}
							</div>
						{/if}
						{#if currentRound > 0}
							<div class="text-sm text-indigo-600 mt-2">
								<span class="font-medium">Vote transfers this round:</span>
								{#each candidates as transferCandidate}
									{@const transferred = getTransferredVotes(transferCandidate.id)}
									{#if transferred > 0}
										<div class="ml-2">• {transferCandidate.name}: +{transferred} votes</div>
									{/if}
								{/each}
							</div>
						{/if}
					{/if}
					{#if currentRoundData.winner}
						<div class="text-green-600 font-medium">
							🏆 {typeof currentRoundData.winner === 'string' 
								? candidates.find(c => c.id === currentRoundData.winner)?.name 
								: currentRoundData.winner.name} wins with majority!
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.candidate-container {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	
	.candidate-card {
		/* Remove transform transitions - let FLIP handle the movement */
		transition: background-color 0.3s ease, border-color 0.3s ease, opacity 0.3s ease;
	}
</style>