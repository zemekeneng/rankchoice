<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte.js';
	import { apiClient } from '$lib/api/client.js';
	import type { CreatePollForm, CreateCandidateForm, FormErrors } from '$lib/types.js';

	// Redirect if not authenticated (wait for auth to load first)
	$effect(() => {
		if (!authStore.isLoading && !authStore.isAuthenticated) {
			goto('/login');
		}
	});

	// Form state
	let pollForm = $state<CreatePollForm>({
		title: '',
		description: '',
		pollType: 'single_winner',
		numWinners: 1,
		opensAt: '',
		closesAt: '',
		isPublic: false,
		registrationRequired: false,
		candidates: [
			{ name: '', description: '' },
			{ name: '', description: '' }
		]
	});

	let errors = $state<FormErrors>({});
	let isSubmitting = $state(false);
	let showPreview = $state(false);
	let submitError = $state<string | null>(null);

	// Add default opens at time (now + 1 hour)
	onMount(() => {
		const now = new Date();
		const oneHourLater = new Date(now.getTime() + 60 * 60 * 1000);
		const tomorrow = new Date(now.getTime() + 24 * 60 * 60 * 1000);
		
		pollForm.opensAt = oneHourLater.toISOString().slice(0, 16);
		pollForm.closesAt = tomorrow.toISOString().slice(0, 16);
	});

	// Validation functions
	function validateTitle(title: string): string[] {
		const titleErrors: string[] = [];
		
		if (!title.trim()) {
			titleErrors.push('Poll title is required');
		} else if (title.trim().length < 3) {
			titleErrors.push('Title must be at least 3 characters');
		} else if (title.trim().length > 500) {
			titleErrors.push('Title must be less than 500 characters');
		}
		
		return titleErrors;
	}

	function validateDescription(description: string): string[] {
		const descErrors: string[] = [];
		
		if (description && description.length > 1000) {
			descErrors.push('Description must be less than 1000 characters');
		}
		
		return descErrors;
	}

	function validateDates(opensAt: string, closesAt: string): { opensAt: string[]; closesAt: string[] } {
		const now = new Date();
		const openDate = opensAt ? new Date(opensAt) : null;
		const closeDate = closesAt ? new Date(closesAt) : null;
		
		const opensAtErrors: string[] = [];
		const closesAtErrors: string[] = [];

		if (openDate && openDate < now) {
			opensAtErrors.push('Opening time must be in the future');
		}

		if (closeDate && closeDate < now) {
			closesAtErrors.push('Closing time must be in the future');
		}

		if (openDate && closeDate && closeDate <= openDate) {
			closesAtErrors.push('Closing time must be after opening time');
		}

		return { opensAt: opensAtErrors, closesAt: closesAtErrors };
	}

	function validateCandidates(candidates: CreateCandidateForm[]): string[] {
		const candidateErrors: string[] = [];
		
		const validCandidates = candidates.filter(c => c.name.trim());
		
		if (validCandidates.length < 2) {
			candidateErrors.push('At least 2 candidates are required');
		}

		const names = validCandidates.map(c => c.name.trim().toLowerCase());
		const uniqueNames = new Set(names);
		
		if (names.length !== uniqueNames.size) {
			candidateErrors.push('Candidate names must be unique');
		}

		// Check individual candidate validations
		candidates.forEach((candidate, index) => {
			if (candidate.name.trim() && candidate.name.trim().length > 500) {
				candidateErrors.push(`Candidate ${index + 1} name too long (max 500 characters)`);
			}
			if (candidate.description && candidate.description.length > 1000) {
				candidateErrors.push(`Candidate ${index + 1} description too long (max 1000 characters)`);
			}
		});

		return candidateErrors;
	}

	function validateNumWinners(pollType: string, numWinners: number, candidates: CreateCandidateForm[]): string[] {
		const errors: string[] = [];
		const validCandidates = candidates.filter(c => c.name.trim()).length;

		if (pollType === 'multi_winner') {
			if (numWinners < 1) {
				errors.push('Number of winners must be at least 1');
			} else if (numWinners >= validCandidates) {
				errors.push('Number of winners must be less than number of candidates');
			}
		}

		return errors;
	}

	// Pure validation function (no state mutation)
	function checkFormValid(): boolean {
		const dateValidation = validateDates(pollForm.opensAt || '', pollForm.closesAt || '');
		
		const formErrors = {
			title: validateTitle(pollForm.title),
			description: validateDescription(pollForm.description),
			opensAt: dateValidation.opensAt,
			closesAt: dateValidation.closesAt,
			candidates: validateCandidates(pollForm.candidates),
			numWinners: validateNumWinners(pollForm.pollType, pollForm.numWinners, pollForm.candidates)
		};

		return Object.values(formErrors).every(fieldErrors => fieldErrors.length === 0);
	}

	// Validate and update errors state
	function validateForm(): boolean {
		const dateValidation = validateDates(pollForm.opensAt || '', pollForm.closesAt || '');
		
		errors = {
			title: validateTitle(pollForm.title),
			description: validateDescription(pollForm.description),
			opensAt: dateValidation.opensAt,
			closesAt: dateValidation.closesAt,
			candidates: validateCandidates(pollForm.candidates),
			numWinners: validateNumWinners(pollForm.pollType, pollForm.numWinners, pollForm.candidates)
		};

		return Object.values(errors).every(fieldErrors => fieldErrors.length === 0);
	}

	// Derived form validity (safe for template use)
	const isFormValid = $derived(checkFormValid());

	// Candidate management
	function addCandidate() {
		pollForm.candidates = [...pollForm.candidates, { name: '', description: '' }];
	}

	function removeCandidate(index: number) {
		if (pollForm.candidates.length > 2) {
			pollForm.candidates = pollForm.candidates.filter((_, i) => i !== index);
		}
	}

	function moveCandidateUp(index: number) {
		if (index > 0) {
			const newCandidates = [...pollForm.candidates];
			[newCandidates[index - 1], newCandidates[index]] = [newCandidates[index], newCandidates[index - 1]];
			pollForm.candidates = newCandidates;
		}
	}

	function moveCandidateDown(index: number) {
		if (index < pollForm.candidates.length - 1) {
			const newCandidates = [...pollForm.candidates];
			[newCandidates[index], newCandidates[index + 1]] = [newCandidates[index + 1], newCandidates[index]];
			pollForm.candidates = newCandidates;
		}
	}

	// Handle poll type change
	function handlePollTypeChange() {
		if (pollForm.pollType === 'single_winner') {
			pollForm.numWinners = 1;
		} else {
			pollForm.numWinners = Math.min(2, pollForm.candidates.filter(c => c.name.trim()).length - 1);
		}
	}

	// Clear field error when user starts typing
	function clearFieldError(field: string) {
		if (errors[field]) {
			errors[field] = [];
		}
	}

	// Handle form submission
	async function handleSubmit(event: Event) {
		event.preventDefault();
		
		if (!validateForm()) {
			return;
		}

		isSubmitting = true;
		submitError = null;

		try {
			// Filter out empty candidates
			const validCandidates = pollForm.candidates.filter(c => c.name.trim());
			
			const pollData: CreatePollForm = {
				...pollForm,
				candidates: validCandidates.map(c => ({
					name: c.name.trim(),
					description: c.description?.trim() || ''
				}))
			};

			const createdPoll = await apiClient.createPoll(pollData);
			
			// Redirect back to dashboard to show the new poll
			await goto('/dashboard?created=true', { invalidateAll: true });
		} catch (error) {
			console.error('Poll creation error:', error);
			
			// Show user-friendly error message
			if (error instanceof Error) {
				submitError = `Failed to create poll: ${error.message}`;
			} else {
				submitError = 'Failed to create poll. Please check your connection and try again.';
			}
		} finally {
			isSubmitting = false;
		}
	}

	// Preview functionality
	function togglePreview() {
		if (isFormValid) {
			showPreview = !showPreview;
		}
	}

	// Get valid candidates for preview
	const validCandidates = $derived(pollForm.candidates.filter(c => c.name.trim()));
</script>

<svelte:head>
	<title>Create Poll - RankChoice</title>
	<meta name="description" content="Create a new ranked-choice voting poll" />
</svelte:head>

<div class="max-w-4xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
	<!-- Header -->
	<div class="mb-8">
		<div class="flex items-center">
			<a href="/dashboard" class="text-indigo-600 hover:text-indigo-500 mr-4">
				<svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
					<path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
				</svg>
			</a>
			<div>
				<h1 class="text-3xl font-bold text-gray-900">Create New Poll</h1>
				<p class="mt-1 text-sm text-gray-500">
					Set up a ranked-choice voting poll with candidates and settings
				</p>
			</div>
		</div>
	</div>

	<!-- Preview Modal -->
	{#if showPreview}
		<div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
			<div class="relative top-20 mx-auto p-5 border w-11/12 md:w-3/4 lg:w-1/2 shadow-lg rounded-md bg-white">
				<div class="mt-3">
					<div class="flex items-center justify-between mb-4">
						<h3 class="text-lg font-medium text-gray-900">Poll Preview</h3>
						<button data-testid="close-preview-btn" onclick={() => showPreview = false} class="text-gray-400 hover:text-gray-600">
							<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
							</svg>
						</button>
					</div>
					<div class="space-y-4">
						<div>
							<h2 class="text-xl font-semibold text-gray-900">{pollForm.title}</h2>
							{#if pollForm.description}
								<p class="mt-2 text-gray-600">{pollForm.description}</p>
							{/if}
						</div>
						<div class="grid grid-cols-2 gap-4 text-sm">
							<div>
								<span class="font-medium">Type:</span> 
								{pollForm.pollType === 'single_winner' ? 'Single Winner' : `Multi Winner (${pollForm.numWinners})`}
							</div>
							<div>
								<span class="font-medium">Visibility:</span> 
								{pollForm.isPublic ? 'Public' : 'Private'}
							</div>
							{#if pollForm.opensAt}
								<div>
									<span class="font-medium">Opens:</span> 
									{new Date(pollForm.opensAt).toLocaleString()}
								</div>
							{/if}
							{#if pollForm.closesAt}
								<div>
									<span class="font-medium">Closes:</span> 
									{new Date(pollForm.closesAt).toLocaleString()}
								</div>
							{/if}
						</div>
						<div>
							<h4 class="font-medium text-gray-900 mb-2">Candidates ({validCandidates.length})</h4>
							<ol class="space-y-2">
								{#each validCandidates as candidate, index}
									<li class="flex items-start">
										<span class="bg-indigo-100 text-indigo-800 text-xs font-medium mr-2 px-2.5 py-0.5 rounded-full">{index + 1}</span>
										<div>
											<div class="font-medium">{candidate.name}</div>
											{#if candidate.description}
												<div class="text-sm text-gray-500">{candidate.description}</div>
											{/if}
										</div>
									</li>
								{/each}
							</ol>
						</div>
					</div>
					<div class="flex justify-end space-x-3 mt-6">
						<button
							onclick={() => showPreview = false}
							class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
						>
							Edit
						</button>
						<button
							data-testid="create-poll-from-preview-btn"
							onclick={handleSubmit}
							disabled={isSubmitting}
							class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 disabled:opacity-50"
						>
							{#if isSubmitting}
								Creating...
							{:else}
								Create Poll
							{/if}
						</button>
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Main Form -->
	<form onsubmit={handleSubmit} class="space-y-8">
		<!-- Basic Information -->
		<div class="bg-white shadow px-4 py-5 sm:rounded-lg sm:p-6">
			<div class="md:grid md:grid-cols-3 md:gap-6">
				<div class="md:col-span-1">
					<h3 class="text-lg font-medium leading-6 text-gray-900">Basic Information</h3>
					<p class="mt-1 text-sm text-gray-500">
						Give your poll a clear title and description that helps voters understand what they're deciding.
					</p>
				</div>
				<div class="mt-5 md:mt-0 md:col-span-2">
					<div class="space-y-6">
						<!-- Title -->
						<div>
							<label for="title" class="block text-sm font-medium text-gray-700">
								Poll Title *
							</label>
							<input
								type="text"
								id="title"
								data-testid="poll-title-input"
								bind:value={pollForm.title}
								oninput={() => clearFieldError('title')}
								class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								class:border-red-300={errors.title?.length > 0}
								placeholder="e.g., Best Programming Language 2024"
								required
							/>
							{#if errors.title?.length > 0}
								<div class="mt-1 text-red-600 text-sm">
									{#each errors.title as error}
										<p>{error}</p>
									{/each}
								</div>
							{/if}
						</div>

						<!-- Description -->
						<div>
							<label for="description" class="block text-sm font-medium text-gray-700">
								Description (Optional)
							</label>
							<textarea
								id="description"
								data-testid="poll-description-input"
								rows="3"
								bind:value={pollForm.description}
								oninput={() => clearFieldError('description')}
								class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
								class:border-red-300={errors.description?.length > 0}
								placeholder="Provide additional context about this poll..."
							></textarea>
							{#if errors.description?.length > 0}
								<div class="mt-1 text-red-600 text-sm">
									{#each errors.description as error}
										<p>{error}</p>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Poll Settings -->
		<div class="bg-white shadow px-4 py-5 sm:rounded-lg sm:p-6">
			<div class="md:grid md:grid-cols-3 md:gap-6">
				<div class="md:col-span-1">
					<h3 class="text-lg font-medium leading-6 text-gray-900">Poll Settings</h3>
					<p class="mt-1 text-sm text-gray-500">
						Configure how your poll works and when it's available for voting.
					</p>
				</div>
				<div class="mt-5 md:mt-0 md:col-span-2">
					<div class="space-y-6">
						<!-- Poll Type -->
						<div>
							<label class="block text-sm font-medium text-gray-700">
								Poll Type *
							</label>
							<div class="mt-2 space-y-3">
								<label class="flex items-center">
									<input
										type="radio"
										name="pollType"
										data-testid="single-winner-radio"
										bind:group={pollForm.pollType}
										value="single_winner"
										onchange={handlePollTypeChange}
										class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
									/>
									<span class="ml-3">
										<span class="block text-sm font-medium text-gray-700">Single Winner</span>
										<span class="block text-sm text-gray-500">One candidate wins (needs >50% after eliminations)</span>
									</span>
								</label>
								<label class="flex items-center">
									<input
										type="radio"
										name="pollType"
										data-testid="multi-winner-radio"
										bind:group={pollForm.pollType}
										value="multi_winner"
										onchange={handlePollTypeChange}
										class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
									/>
									<span class="ml-3">
										<span class="block text-sm font-medium text-gray-700">Multi Winner</span>
										<span class="block text-sm text-gray-500">Multiple candidates can win (proportional representation)</span>
									</span>
								</label>
							</div>
						</div>

						<!-- Number of Winners (for multi-winner) -->
						{#if pollForm.pollType === 'multi_winner'}
							<div>
								<label for="numWinners" class="block text-sm font-medium text-gray-700">
									Number of Winners
								</label>
								<input
									type="number"
									id="numWinners"
									data-testid="num-winners-input"
									bind:value={pollForm.numWinners}
									min="2"
									max={Math.max(2, validCandidates.length - 1)}
									oninput={() => clearFieldError('numWinners')}
									class="mt-1 block w-20 border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
									class:border-red-300={errors.numWinners?.length > 0}
								/>
								{#if errors.numWinners?.length > 0}
									<div class="mt-1 text-red-600 text-sm">
										{#each errors.numWinners as error}
											<p>{error}</p>
										{/each}
									</div>
								{/if}
							</div>
						{/if}

						<!-- Timing -->
						<div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
							<div>
								<label for="opensAt" class="block text-sm font-medium text-gray-700">
									Opens At (Optional)
								</label>
								<input
									type="datetime-local"
									id="opensAt"
									bind:value={pollForm.opensAt}
									oninput={() => clearFieldError('opensAt')}
									class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
									class:border-red-300={errors.opensAt?.length > 0}
								/>
								{#if errors.opensAt?.length > 0}
									<div class="mt-1 text-red-600 text-sm">
										{#each errors.opensAt as error}
											<p>{error}</p>
										{/each}
									</div>
								{/if}
							</div>

							<div>
								<label for="closesAt" class="block text-sm font-medium text-gray-700">
									Closes At (Optional)
								</label>
								<input
									type="datetime-local"
									id="closesAt"
									bind:value={pollForm.closesAt}
									oninput={() => clearFieldError('closesAt')}
									class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
									class:border-red-300={errors.closesAt?.length > 0}
								/>
								{#if errors.closesAt?.length > 0}
									<div class="mt-1 text-red-600 text-sm">
										{#each errors.closesAt as error}
											<p>{error}</p>
										{/each}
									</div>
								{/if}
							</div>
						</div>

						<!-- Visibility Options -->
						<div class="space-y-3">
							<label class="flex items-center">
								<input
									type="checkbox"
									data-testid="poll-public-checkbox"
									bind:checked={pollForm.isPublic}
									class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
								/>
								<span class="ml-3">
									<span class="block text-sm font-medium text-gray-700">Make poll public</span>
									<span class="block text-sm text-gray-500">Anyone with the link can view results (they still need voting links to vote)</span>
								</span>
							</label>

							<label class="flex items-center">
								<input
									type="checkbox"
									bind:checked={pollForm.registrationRequired}
									class="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded"
								/>
								<span class="ml-3">
									<span class="block text-sm font-medium text-gray-700">Require voter registration</span>
									<span class="block text-sm text-gray-500">Voters must provide email and register before voting</span>
								</span>
							</label>
						</div>
					</div>
				</div>
			</div>
		</div>

		<!-- Candidates -->
		<div class="bg-white shadow px-4 py-5 sm:rounded-lg sm:p-6">
			<div class="md:grid md:grid-cols-3 md:gap-6">
				<div class="md:col-span-1">
					<h3 class="text-lg font-medium leading-6 text-gray-900">Candidates</h3>
					<p class="mt-1 text-sm text-gray-500">
						Add the candidates or options that voters will rank. Each candidate needs a name, and you can optionally add a description. You need at least 2 candidates.
					</p>
					<div class="mt-3 text-xs text-gray-400">
						💡 Use the up/down arrows to reorder candidates
					</div>
				</div>
				<div class="mt-5 md:mt-0 md:col-span-2">
					<div class="space-y-4">
						{#each pollForm.candidates as candidate, index}
							<div class="border border-gray-200 rounded-lg p-4">
								<div class="flex justify-between items-start mb-3">
									<h4 class="text-sm font-medium text-gray-900">Candidate {index + 1}</h4>
									<div class="flex space-x-2">
										<!-- Move buttons -->
										<button
											type="button"
											data-testid="move-candidate-up-{index}"
											onclick={() => moveCandidateUp(index)}
											disabled={index === 0}
											class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50"
											title="Move up"
										>
											<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
											</svg>
										</button>
										<button
											type="button"
											data-testid="move-candidate-down-{index}"
											onclick={() => moveCandidateDown(index)}
											disabled={index === pollForm.candidates.length - 1}
											class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50"
											title="Move down"
										>
											<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
											</svg>
										</button>
										<!-- Delete button -->
										<button
											type="button"
											data-testid="remove-candidate-{index}"
											onclick={() => removeCandidate(index)}
											disabled={pollForm.candidates.length <= 2}
											class="p-1 text-red-400 hover:text-red-600 disabled:opacity-50"
											title="Remove candidate"
										>
											<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
											</svg>
										</button>
									</div>
								</div>
								
								<div class="space-y-3">
									<div>
										<label for="candidate-name-{index}" class="block text-xs font-medium text-gray-700">
											Name *
										</label>
										<input
											type="text"
											id="candidate-name-{index}"
											data-testid="candidate-name-{index}"
											bind:value={candidate.name}
											oninput={() => clearFieldError('candidates')}
											class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 text-sm"
											placeholder="Candidate name"
										/>
									</div>
									
									<div>
										<label for="candidate-desc-{index}" class="block text-xs font-medium text-gray-700">
											Description (Optional)
										</label>
										<textarea
											id="candidate-desc-{index}"
											rows="2"
											bind:value={candidate.description}
											class="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 text-sm"
											placeholder="Brief description of this candidate or option"
										></textarea>
									</div>
								</div>
							</div>
						{/each}

						<!-- Add Candidate Button -->
						<button
							type="button"
							data-testid="add-candidate-btn"
							onclick={addCandidate}
							class="w-full border-2 border-dashed border-gray-300 rounded-lg p-4 text-gray-600 hover:border-indigo-300 hover:text-indigo-600 hover:bg-indigo-50 transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
						>
							<svg class="mx-auto h-6 w-6 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
							</svg>
							<div class="text-sm font-medium">Add Another Candidate</div>
							<div class="text-xs text-gray-500 mt-1">Click to add more options for voters to rank</div>
						</button>

						<!-- Candidates Validation Errors -->
						{#if errors.candidates?.length > 0}
							<div class="text-red-600 text-sm">
								{#each errors.candidates as error}
									<p>{error}</p>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>
		</div>

		<!-- Submit Error -->
		{#if submitError}
			<div class="bg-red-50 border border-red-200 rounded-md p-4" data-testid="poll-creation-error">
				<div class="flex">
					<div class="flex-shrink-0">
						<svg class="h-5 w-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
						</svg>
					</div>
					<div class="ml-3">
						<h3 class="text-sm font-medium text-red-800">Poll Creation Failed</h3>
						<div class="mt-2 text-sm text-red-700">
							<p>{submitError}</p>
						</div>
					</div>
					<div class="ml-auto pl-3">
						<div class="-mx-1.5 -my-1.5">
							<button
								data-testid="dismiss-error-btn"
								onclick={() => submitError = null}
								class="inline-flex bg-red-50 rounded-md p-1.5 text-red-500 hover:bg-red-100"
							>
								<svg class="h-5 w-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
									<path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
								</svg>
							</button>
						</div>
					</div>
				</div>
			</div>
		{/if}

		<!-- Actions -->
		<div class="flex justify-end space-x-3" data-testid="form-actions">
			<button
				type="button"
				data-testid="cancel-poll-btn"
				onclick={() => goto('/dashboard')}
				class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50"
			>
				Cancel
			</button>
			
			<button
				type="button"
				data-testid="preview-poll-btn"
				onclick={togglePreview}
				disabled={!isFormValid}
				class="bg-gray-600 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white hover:bg-gray-700 disabled:opacity-50"
			>
				Preview
			</button>
			
			<button
				type="submit"
				data-testid="create-poll-submit-btn"
				disabled={isSubmitting}
				class="bg-indigo-600 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50"
			>
				{#if isSubmitting}
					<svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					Creating Poll...
				{:else}
					Create Poll
				{/if}
			</button>
		</div>
	</form>
</div> 