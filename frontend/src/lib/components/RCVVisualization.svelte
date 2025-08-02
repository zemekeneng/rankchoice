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
		<div class="px-6 py-6 relative">
			<!-- 50% majority line spanning all candidates -->
			<div 
				class="absolute top-0 bottom-0 w-0.5 bg-yellow-500 z-40 pointer-events-none"
				style="left: calc(50% + 1.5rem)"
			>
				<div class="absolute -top-2 -left-12 text-xs text-yellow-600 font-medium whitespace-nowrap">
					Over 50% to win
				</div>
			</div>
			
			<div class="space-y-4">
			{#each candidates as candidate}
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
				
				<div 
					class="relative p-4 rounded-lg border-2 transition-all duration-500"
					class:bg-green-50={winner}
					class:border-green-300={winner}
					class:bg-red-50={eliminated}
					class:border-red-300={eliminated}
					class:bg-gray-50={shouldStripe && !eliminated}
					class:border-gray-300={shouldStripe && !eliminated}
					class:bg-white={!shouldStripe && !winner}
					class:border-gray-200={!shouldStripe && !winner}
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
								{:else if shouldStripe}
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
						<div class="w-full bg-gray-200 h-6 overflow-hidden" style="border-radius: 0.375rem">
							{#if !shouldStripe}
								<!-- Base votes (from previous rounds) -->
								{#if basePercentage > 0}
									<div 
										class="h-6 transition-all duration-700 ease-out absolute left-0 top-0 z-10 flex items-center justify-center"
										style="width: {basePercentage}%; background-color: {color}; border-radius: {transferredPercentage > 0 ? '0.375rem 0 0 0.375rem' : '0.375rem'}"
									>
										<!-- Round label for base votes -->
										{#if currentRound === 0}
											{#if basePercentage > 15}
												<span class="text-xs font-medium text-white drop-shadow">Round 1</span>
											{:else if basePercentage > 8}
												<span class="text-xs font-medium text-white drop-shadow">1</span>
											{/if}
										{:else if currentRound > 0 && basePercentage > 15}
											<span class="text-xs font-medium text-white drop-shadow">Round 1</span>
										{:else if currentRound > 0 && basePercentage > 8}
											<span class="text-xs font-medium text-white drop-shadow">1</span>
										{/if}
									</div>
								{/if}
								
								<!-- Transferred votes (darker shade) -->
								{#if transferredPercentage > 0}
									<div 
										class="h-6 transition-all duration-700 ease-out absolute top-0 z-20 flex items-center justify-center"
										style="left: {basePercentage}%; width: {transferredPercentage}%; background-color: {darkerColor}; border-radius: 0 0.375rem 0.375rem 0"
									>
										<!-- Round label for transferred votes -->
										{#if transferredPercentage > 15}
											<span class="text-xs font-medium text-white drop-shadow">Round {currentRound + 1}</span>
										{:else if transferredPercentage > 8}
											<span class="text-xs font-medium text-white drop-shadow">{currentRound + 1}</span>
										{/if}
									</div>
								{/if}
								
								<!-- Remove majority threshold indicator -->
							{:else}
								<!-- Eliminated or inactive candidate - show original color with gray stripes -->
								{@const showWidth = eliminated ? percentage : (currentRound === 0 ? percentage : getPreviousRoundVotes(candidate.id) / totalVotes * 100)}
								<div 
									class="h-6 transition-all duration-700 ease-out absolute left-0 top-0 z-10 flex items-center justify-center"
									style="width: {showWidth}%; background-color: {color}; background-image: repeating-linear-gradient(45deg, rgba(107, 114, 128, 0.8) 0px, rgba(107, 114, 128, 0.8) 4px, transparent 4px, transparent 8px); border-radius: 0.375rem"
								>
									{#if showWidth > 15}
										<span class="text-xs font-medium text-white drop-shadow">Round 1</span>
									{:else if showWidth > 8}
										<span class="text-xs font-medium text-white drop-shadow">1</span>
									{/if}
								</div>
							{/if}
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
							🏆 {candidates.find(c => c.id === currentRoundData.winner)?.name} wins with majority!
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>