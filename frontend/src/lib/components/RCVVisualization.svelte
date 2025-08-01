<script lang="ts">
	import type { RCVRound, Candidate } from '$lib/types.js';

	interface Props {
		rounds: RCVRound[];
		candidates: Candidate[];
		totalVotes: number;
	}

	let { rounds, candidates, totalVotes }: Props = $props();

	// State management
	let currentRound = $state(0);
	let isPlaying = $state(false);
	let playbackSpeed = $state(1000); // ms between rounds

	// Derived state
	let maxRound = $derived(rounds.length - 1);
	let currentRoundData = $derived(rounds[currentRound] || null);
	let majorityThreshold = $derived(Math.floor(totalVotes / 2) + 1);

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
		if (!currentRoundData) return false;
		return currentRoundData.eliminated === candidateId;
	}

	// Check if candidate won in current round
	function isWinner(candidateId: string): boolean {
		if (!currentRoundData) return false;
		return currentRoundData.winner === candidateId;
	}

	// Check if candidate is still active (not eliminated in previous rounds)
	function isActive(candidateId: string): boolean {
		for (let i = 0; i < currentRound; i++) {
			if (rounds[i].eliminated === candidateId) {
				return false;
			}
		}
		return true;
	}

	// Navigation functions
	function nextRound() {
		if (currentRound < maxRound) {
			currentRound++;
		}
	}

	function prevRound() {
		if (currentRound > 0) {
			currentRound--;
		}
	}

	function goToRound(round: number) {
		currentRound = Math.max(0, Math.min(round, maxRound));
	}

	// Auto-play functionality
	let playInterval: number | null = null;

	function togglePlay() {
		isPlaying = !isPlaying;
		
		if (isPlaying) {
			playInterval = setInterval(() => {
				if (currentRound < maxRound) {
					nextRound();
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

	function reset() {
		isPlaying = false;
		if (playInterval) {
			clearInterval(playInterval);
			playInterval = null;
		}
		currentRound = 0;
	}

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
						• {candidates.find(c => c.id === currentRoundData?.eliminated)?.name} eliminated
					{:else if currentRoundData?.winner}
						• {candidates.find(c => c.id === currentRoundData?.winner)?.name} wins!
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
	<div class="px-6 py-6">
		<div class="space-y-4">
			{#each candidates as candidate}
				{@const votes = getVoteCount(candidate.id)}
				{@const percentage = getVotePercentage(candidate.id)}
				{@const active = isActive(candidate.id)}
				{@const eliminated = isEliminated(candidate.id)}
				{@const winner = isWinner(candidate.id)}
				{@const color = candidateColors[candidate.id]}
				
				<div 
					class="relative p-4 rounded-lg border-2 transition-all duration-500"
					class:bg-green-50={winner}
					class:border-green-300={winner}
					class:bg-red-50={eliminated}
					class:border-red-300={eliminated}
					class:bg-gray-50={!active && !eliminated}
					class:border-gray-300={!active && !eliminated}
					class:bg-white={active && !winner && !eliminated}
					class:border-gray-200={active && !winner && !eliminated}
					class:opacity-60={!active}
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
								{#if eliminated}
									<span class="text-xs text-red-600 font-medium">Eliminated</span>
								{:else if winner}
									<span class="text-xs text-green-600 font-medium">🏆 Winner!</span>
								{:else if !active}
									<span class="text-xs text-gray-500">Previously eliminated</span>
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
						<div class="w-full bg-gray-200 rounded-full h-6 overflow-hidden">
							<div 
								class="h-6 rounded-full transition-all duration-700 ease-out relative"
								style="width: {percentage}%; background-color: {color}"
							>
								<!-- Majority threshold indicator -->
								{#if percentage >= (majorityThreshold / totalVotes) * 100}
									<div class="absolute inset-0 bg-yellow-400 opacity-30 animate-pulse"></div>
								{/if}
							</div>
						</div>
						
						<!-- Majority threshold line -->
						<div 
							class="absolute top-0 h-6 w-0.5 bg-yellow-500"
							style="left: {(majorityThreshold / totalVotes) * 100}%"
						>
							<div class="absolute -top-1 -left-6 text-xs text-yellow-600 font-medium">
								Majority
							</div>
						</div>
					</div>

					<!-- Vote transfer animation indicator -->
					{#if eliminated && currentRound < maxRound}
						<div class="absolute inset-0 pointer-events-none">
							<div class="absolute inset-0 bg-red-200 opacity-50 animate-pulse rounded-lg"></div>
							<div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
								<div class="text-red-600 font-medium text-sm animate-bounce">
									Votes transferring...
								</div>
							</div>
						</div>
					{/if}
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
							{candidates.find(c => c.id === currentRoundData.eliminated)?.name} eliminated (lowest votes)
						</div>
					{/if}
					{#if currentRoundData.winner}
						<div class="text-green-600 font-medium">
							🏆 {candidates.find(c => c.id === currentRoundData.winner)?.name} wins with majority!
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>