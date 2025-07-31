<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte.js';
	import { goto } from '$app/navigation';

	// Navigation functions
	function goToDashboard() {
		goto('/dashboard');
	}

	function goToLogin() {
		goto('/login');
	}

	function goToRegister() {
		goto('/register');
	}
</script>

<svelte:head>
	<title>RankChoice.app - Modern Ranked-Choice Voting Platform</title>
	<meta name="description" content="Create and manage ranked-choice voting polls with real-time results and secure ballot distribution." />
</svelte:head>

<!-- Hero Section -->
<div class="bg-indigo-700">
	<div class="max-w-7xl mx-auto py-16 px-4 sm:py-24 sm:px-6 lg:px-8">
		<div class="text-center">
			<h1 class="text-4xl font-extrabold tracking-tight text-white sm:text-5xl lg:text-6xl">
				🗳️ RankChoice
			</h1>
			<p class="mt-6 max-w-2xl mx-auto text-xl text-indigo-100">
				The modern platform for ranked-choice voting. Create polls, distribute ballots, and visualize results with transparent RCV tabulation.
			</p>
			<div class="mt-10 flex justify-center space-x-4">
				{#if authStore.isAuthenticated}
					<button
						onclick={goToDashboard}
						class="bg-white text-indigo-600 border-2 border-white hover:bg-indigo-50 px-8 py-3 text-lg font-medium rounded-lg shadow-lg transition-colors"
					>
						Go to Dashboard
					</button>
				{:else}
					<button
						onclick={goToRegister}
						class="bg-white text-indigo-600 border-2 border-white hover:bg-indigo-50 px-8 py-3 text-lg font-medium rounded-lg shadow-lg transition-colors"
					>
						Get Started
					</button>
					<button
						onclick={goToLogin}
						class="bg-transparent text-white border-2 border-white hover:bg-white hover:text-indigo-600 px-8 py-3 text-lg font-medium rounded-lg shadow-lg transition-colors"
					>
						Sign In
					</button>
				{/if}
			</div>
		</div>
	</div>
</div>

<!-- Features Section -->
<div class="py-16 bg-white">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="text-center">
			<h2 class="text-3xl font-extrabold text-gray-900 sm:text-4xl">
				Why Choose RankChoice?
			</h2>
			<p class="mt-4 max-w-2xl mx-auto text-xl text-gray-500">
				Built for transparency, security, and ease of use
			</p>
		</div>

		<div class="mt-16">
			<div class="grid grid-cols-1 gap-8 md:grid-cols-2 lg:grid-cols-3">
				<!-- Feature 1 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white mx-auto">
						<svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
						</svg>
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Easy Poll Creation</h3>
					<p class="mt-2 text-base text-gray-500">
						Create ranked-choice voting polls in minutes with our intuitive interface.
					</p>
				</div>

				<!-- Feature 2 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white mx-auto">
						<svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
						</svg>
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Secure Voting</h3>
					<p class="mt-2 text-base text-gray-500">
						Anonymous, secure ballots with unique tokens and fraud prevention.
					</p>
				</div>

				<!-- Feature 3 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white mx-auto">
						<svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
						</svg>
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Real-time Results</h3>
					<p class="mt-2 text-base text-gray-500">
						Watch RCV tabulation in real-time with transparent round-by-round breakdowns.
					</p>
				</div>

				<!-- Feature 4 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white mx-auto">
						<svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2M7 4h10M7 4l-2 14h14l-2-14M11 9v6m2-6v6" />
						</svg>
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Drag & Drop Ranking</h3>
					<p class="mt-2 text-base text-gray-500">
						Intuitive drag-and-drop interface makes ranking candidates effortless.
					</p>
				</div>

				<!-- Feature 5 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white mx-auto">
						<svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
						</svg>
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Email Distribution</h3>
					<p class="mt-2 text-base text-gray-500">
						Send personalized voting links directly to participants via email.
					</p>
				</div>

				<!-- Feature 6 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-12 w-12 rounded-md bg-indigo-500 text-white mx-auto">
						<svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z" />
						</svg>
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Verified Results</h3>
					<p class="mt-2 text-base text-gray-500">
						Every vote comes with a receipt and verification code for transparency.
					</p>
				</div>
			</div>
		</div>
	</div>
</div>

<!-- How it Works Section -->
<div class="py-16 bg-gray-50">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="text-center">
			<h2 class="text-3xl font-extrabold text-gray-900 sm:text-4xl">
				How Ranked-Choice Voting Works
			</h2>
			<p class="mt-4 max-w-2xl mx-auto text-xl text-gray-500">
				Simple, fair, and transparent election method
			</p>
		</div>

		<div class="mt-16">
			<div class="grid grid-cols-1 gap-8 md:grid-cols-3">
				<!-- Step 1 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-16 w-16 rounded-full bg-indigo-100 text-indigo-600 mx-auto text-2xl font-bold">
						1
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Voters Rank Candidates</h3>
					<p class="mt-2 text-base text-gray-500">
						Instead of choosing just one candidate, voters rank their choices in order of preference.
					</p>
				</div>

				<!-- Step 2 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-16 w-16 rounded-full bg-indigo-100 text-indigo-600 mx-auto text-2xl font-bold">
						2
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Count First Choices</h3>
					<p class="mt-2 text-base text-gray-500">
						First choice votes are tallied. If someone has >50%, they win. Otherwise, eliminate the last-place candidate.
					</p>
				</div>

				<!-- Step 3 -->
				<div class="text-center">
					<div class="flex items-center justify-center h-16 w-16 rounded-full bg-indigo-100 text-indigo-600 mx-auto text-2xl font-bold">
						3
					</div>
					<h3 class="mt-4 text-lg font-medium text-gray-900">Redistribute Votes</h3>
					<p class="mt-2 text-base text-gray-500">
						Votes for eliminated candidates transfer to voters' next choices until someone reaches majority.
					</p>
				</div>
			</div>
		</div>
	</div>
</div>

<!-- CTA Section -->
{#if !authStore.isAuthenticated}
	<div class="bg-indigo-700">
		<div class="max-w-2xl mx-auto text-center py-16 px-4 sm:py-20 sm:px-6 lg:px-8">
			<h2 class="text-3xl font-extrabold text-white sm:text-4xl">
				<span class="block">Ready to get started?</span>
				<span class="block">Create your first poll today.</span>
			</h2>
			<p class="mt-4 text-lg leading-6 text-indigo-200">
				Join organizations using RankChoice for fair, transparent elections.
			</p>
			<button
				onclick={goToRegister}
				class="mt-8 w-full inline-flex items-center justify-center px-5 py-3 border border-transparent text-base font-medium rounded-md text-indigo-600 bg-white hover:bg-indigo-50 sm:w-auto"
			>
				Sign up for free
			</button>
		</div>
	</div>
{/if}
