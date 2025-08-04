<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte.js';
	import { apiClient } from '$lib/api/client.js';
	import type { Poll, Candidate, CreatePollForm, CreateCandidateForm, FormErrors } from '$lib/types.js';

	// Get poll ID from URL
	let pollId = $derived($page.params.id);

	// Redirect if not authenticated
	$effect(() => {
		if (!authStore.isLoading && !authStore.isAuthenticated) {
			goto('/login');
		}
	});

	// State
	let poll = $state<Poll | null>(null);
	let candidates = $state<Candidate[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

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
		candidates: []
	});

	// Candidate management state
	let originalCandidates = $state<Candidate[]>([]);
	let candidateChanges = $state<{
		toAdd: CreateCandidateForm[];
		toUpdate: Array<{ id: string; data: CreateCandidateForm }>;
		toDelete: string[];
	}>({
		toAdd: [],
		toUpdate: [],
		toDelete: []
	});

	let errors = $state<FormErrors>({});
	let isSubmitting = $state(false);
	let submitError = $state<string | null>(null);

	// Load existing poll data
	async function loadPoll() {
		if (!pollId || !authStore.isAuthenticated) return;

		try {
			isLoading = true;
			error = null;

			// Load poll details
			poll = await apiClient.getPoll(pollId);
			
			// Load candidates
			candidates = await apiClient.getCandidates(pollId);
			originalCandidates = [...candidates]; // Store original for comparison

			// Populate form with existing data
			console.log('Loading poll data:', poll);
			console.log('Poll isPublic value:', poll.isPublic);
			pollForm = {
				title: poll.title,
				description: poll.description || '',
				pollType: poll.pollType,
				numWinners: poll.numWinners,
				opensAt: poll.opensAt ? new Date(poll.opensAt).toISOString().slice(0, 16) : '',
				closesAt: poll.closesAt ? new Date(poll.closesAt).toISOString().slice(0, 16) : '',
				isPublic: poll.isPublic,
				registrationRequired: poll.registrationRequired,
				candidates: candidates.map(c => ({
					name: c.name,
					description: c.description || '',
					id: c.id // Add ID for tracking
				}))
			};
			console.log('Form isPublic value:', pollForm.isPublic);

			// Reset candidate changes
			candidateChanges = { toAdd: [], toUpdate: [], toDelete: [] };

		} catch (err: any) {
			console.error('Error loading poll:', err);
			if (err.status === 404) {
				error = 'Poll not found';
			} else if (err.status === 403) {
				error = 'You do not have permission to edit this poll';
			} else {
				error = 'Failed to load poll. Please try again.';
			}
		} finally {
			isLoading = false;
		}
	}

	// Load poll on mount
	onMount(() => {
		loadPoll();
	});

	// Validation functions (reused from create poll page)
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
		const dateErrors = { opensAt: [] as string[], closesAt: [] as string[] };
		
		if (opensAt && closesAt) {
			const opensDate = new Date(opensAt);
			const closesDate = new Date(closesAt);
			
			if (closesDate <= opensDate) {
				dateErrors.closesAt.push('Closing time must be after opening time');
			}
		}
		
		return dateErrors;
	}

	function validateCandidates(candidates: CreateCandidateForm[]): string[] {
		const candidateErrors: string[] = [];
		
		if (candidates.length < 2) {
			candidateErrors.push('At least 2 candidates are required');
		}
		
		const validCandidates = candidates.filter(c => c.name.trim());
		if (validCandidates.length < 2) {
			candidateErrors.push('At least 2 candidates must have names');
		}
		
		// Check for duplicate names
		const names = validCandidates.map(c => c.name.trim().toLowerCase());
		const duplicates = names.filter((name, index) => names.indexOf(name) !== index);
		if (duplicates.length > 0) {
			candidateErrors.push('Candidate names must be unique');
		}
		
		return candidateErrors;
	}

	function validateForm(): boolean {
		const newErrors: FormErrors = {};
		
		// Validate title
		const titleErrors = validateTitle(pollForm.title);
		if (titleErrors.length > 0) {
			newErrors.title = titleErrors;
		}
		
		// Validate description
		const descriptionErrors = validateDescription(pollForm.description);
		if (descriptionErrors.length > 0) {
			newErrors.description = descriptionErrors;
		}
		
		// Validate dates
		const dateErrors = validateDates(pollForm.opensAt, pollForm.closesAt);
		if (dateErrors.opensAt.length > 0) {
			newErrors.opensAt = dateErrors.opensAt;
		}
		if (dateErrors.closesAt.length > 0) {
			newErrors.closesAt = dateErrors.closesAt;
		}
		
		// Validate candidates
		const candidateErrors = validateCandidates(pollForm.candidates);
		if (candidateErrors.length > 0) {
			newErrors.candidates = candidateErrors;
		}
		
		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	// Form handlers
	function addCandidate() {
		const newCandidate = { name: '', description: '' };
		pollForm.candidates = [...pollForm.candidates, newCandidate];
		// Track as new candidate to add
		candidateChanges.toAdd = [...candidateChanges.toAdd, newCandidate];
	}

	function removeCandidate(index: number) {
		if (pollForm.candidates.length > 2) {
			const candidateToRemove = pollForm.candidates[index];
			
			// If it has an ID, it's an existing candidate that needs to be deleted
			if (candidateToRemove.id) {
				candidateChanges.toDelete = [...candidateChanges.toDelete, candidateToRemove.id];
				// Remove from update list if it was there
				candidateChanges.toUpdate = candidateChanges.toUpdate.filter(u => u.id !== candidateToRemove.id);
			} else {
				// It's a new candidate, remove from toAdd list
				candidateChanges.toAdd = candidateChanges.toAdd.filter(c => c !== candidateToRemove);
			}
			
			pollForm.candidates = pollForm.candidates.filter((_, i) => i !== index);
		}
	}

	function updateCandidate(index: number, field: 'name' | 'description', value: string) {
		const candidate = pollForm.candidates[index];
		candidate[field] = value;
		
		// If it's an existing candidate (has ID), track as needing update
		if (candidate.id) {
			const existingUpdateIndex = candidateChanges.toUpdate.findIndex(u => u.id === candidate.id);
			if (existingUpdateIndex >= 0) {
				// Update existing change record
				candidateChanges.toUpdate[existingUpdateIndex].data = {
					name: candidate.name,
					description: candidate.description
				};
			} else {
				// Add new change record
				candidateChanges.toUpdate = [...candidateChanges.toUpdate, {
					id: candidate.id,
					data: { name: candidate.name, description: candidate.description }
				}];
			}
		}
		
		// Clear validation errors when user starts typing
		if (errors.candidates) {
			errors = { ...errors, candidates: undefined };
		}
	}

	// Submit form
	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (!validateForm() || !poll) return;

		try {
			isSubmitting = true;
			submitError = null;

			// Create poll update data (excluding candidates) with proper field names for API
			const pollUpdateData = {
				title: pollForm.title,
				description: pollForm.description,
				poll_type: pollForm.pollType,
				num_winners: pollForm.numWinners,
				opens_at: pollForm.opensAt && pollForm.opensAt.trim() ? new Date(pollForm.opensAt).toISOString() : null,
				closes_at: pollForm.closesAt && pollForm.closesAt.trim() ? new Date(pollForm.closesAt).toISOString() : null,
				is_public: pollForm.isPublic,
				registration_required: pollForm.registrationRequired
			};

			// Update poll basic information
			console.log('Sending poll update data:', pollUpdateData);
			await apiClient.updatePoll(poll.id, pollUpdateData);
			console.log('Poll update successful');

			// Handle candidate changes
			console.log('Candidate changes to process:', candidateChanges);
			
			// Process candidate changes sequentially for better error handling
			const results = [];

			// Delete candidates first
			for (const candidateId of candidateChanges.toDelete) {
				console.log('Deleting candidate:', candidateId);
				try {
					await apiClient.deleteCandidate(candidateId);
					results.push({ type: 'delete', id: candidateId, success: true });
				} catch (err) {
					console.error('Failed to delete candidate:', candidateId, err);
					results.push({ type: 'delete', id: candidateId, success: false, error: err });
				}
			}

			// Update existing candidates
			for (const update of candidateChanges.toUpdate) {
				console.log('Updating candidate:', update.id, update.data);
				try {
					await apiClient.updateCandidate(update.id, update.data);
					results.push({ type: 'update', id: update.id, success: true });
				} catch (err) {
					console.error('Failed to update candidate:', update.id, err);
					results.push({ type: 'update', id: update.id, success: false, error: err });
				}
			}

			// Add new candidates
			for (const newCandidate of candidateChanges.toAdd) {
				if (newCandidate.name.trim()) { // Only add if name is not empty
					console.log('Adding new candidate:', newCandidate);
					try {
						await apiClient.addCandidate(poll.id, newCandidate);
						results.push({ type: 'add', data: newCandidate, success: true });
					} catch (err) {
						console.error('Failed to add candidate:', newCandidate, err);
						results.push({ type: 'add', data: newCandidate, success: false, error: err });
					}
				}
			}

			console.log('Candidate operation results:', results);
			
			// Check if any operations failed
			const failedOperations = results.filter(r => !r.success);
			if (failedOperations.length > 0) {
				console.warn('Some candidate operations failed:', failedOperations);
				// Still proceed to navigate, but log the failures
			}

			// Navigate back to poll management page
			goto(`/polls/${poll.id}?updated=true`);

		} catch (err: any) {
			console.error('Error updating poll:', err);
			if (err.status === 400 && err.data?.error?.details) {
				// Handle validation errors from server
				const serverErrors: FormErrors = {};
				err.data.error.details.forEach((detail: any) => {
					const field = detail.path?.[0];
					if (field) {
						if (!serverErrors[field]) serverErrors[field] = [];
						serverErrors[field].push(detail.message);
					}
				});
				errors = serverErrors;
			} else {
				submitError = err.message || 'Failed to update poll. Please try again.';
			}
		} finally {
			isSubmitting = false;
		}
	}

	// Cancel editing
	function handleCancel() {
		goto(`/polls/${pollId}`);
	}
</script>

<svelte:head>
	<title>Edit Poll - {poll?.title || 'Loading...'} - RankChoice</title>
	<meta name="description" content="Edit your ranked-choice voting poll" />
</svelte:head>

<div class="max-w-4xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
	{#if isLoading}
		<div class="flex items-center justify-center min-h-screen">
			<div class="text-center">
				<div class="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600 mx-auto mb-4"></div>
				<p class="text-gray-600">Loading poll for editing...</p>
			</div>
		</div>
	{:else if error}
		<div class="bg-red-50 border border-red-200 rounded-md p-4 mb-6">
			<h2 class="text-red-800 font-semibold">Error Loading Poll</h2>
			<p class="text-red-600">{error}</p>
			<button 
				onclick={() => goto('/dashboard')}
				class="mt-2 text-red-700 underline hover:text-red-800"
			>
				← Back to Dashboard
			</button>
		</div>
	{:else if poll}
		<!-- Header -->
		<div class="mb-8">
			<div class="flex items-center justify-between">
				<div>
					<h1 class="text-3xl font-bold text-gray-900">Edit Poll</h1>
					<p class="mt-2 text-gray-600">Update your ranked-choice voting poll settings and candidates</p>
				</div>
				<button
					onclick={handleCancel}
					class="text-gray-500 hover:text-gray-700 text-sm font-medium"
				>
					← Back to Poll
				</button>
			</div>
		</div>

		<!-- Form -->
		<form onsubmit={handleSubmit} class="space-y-8">
			<!-- Basic Information -->
			<div class="bg-white shadow rounded-lg p-6">
				<h2 class="text-lg font-medium text-gray-900 mb-4">Basic Information</h2>
				
				<div class="grid grid-cols-1 gap-6">
					<!-- Title -->
					<div>
						<label for="title" class="block text-sm font-medium text-gray-700 mb-2">
							Poll Title *
						</label>
						<input
							type="text"
							id="title"
							bind:value={pollForm.title}
							class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
							class:border-red-500={errors.title}
							placeholder="Enter poll title"
							required
						/>
						{#if errors.title}
							<div class="mt-1 text-sm text-red-600">
								{#each errors.title as error}
									<p>{error}</p>
								{/each}
							</div>
						{/if}
					</div>

					<!-- Description -->
					<div>
						<label for="description" class="block text-sm font-medium text-gray-700 mb-2">
							Description (Optional)
						</label>
						<textarea
							id="description"
							bind:value={pollForm.description}
							rows="3"
							class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
							class:border-red-500={errors.description}
							placeholder="Describe your poll (optional)"
						></textarea>
						{#if errors.description}
							<div class="mt-1 text-sm text-red-600">
								{#each errors.description as error}
									<p>{error}</p>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>

			<!-- Poll Settings -->
			<div class="bg-white shadow rounded-lg p-6">
				<h2 class="text-lg font-medium text-gray-900 mb-4">Poll Settings</h2>
				
				<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
					<!-- Poll Type -->
					<div>
						<label for="pollType" class="block text-sm font-medium text-gray-700 mb-2">
							Poll Type
						</label>
						<select
							id="pollType"
							bind:value={pollForm.pollType}
							class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
						>
							<option value="single_winner">Single Winner</option>
							<option value="multi_winner">Multiple Winners</option>
						</select>
					</div>

					<!-- Number of Winners (if multi-winner) -->
					{#if pollForm.pollType === 'multi_winner'}
						<div>
							<label for="numWinners" class="block text-sm font-medium text-gray-700 mb-2">
								Number of Winners
							</label>
							<input
								type="number"
								id="numWinners"
								bind:value={pollForm.numWinners}
								min="2"
								max="10"
								class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
							/>
						</div>
					{/if}

					<!-- Opens At -->
					<div>
						<label for="opensAt" class="block text-sm font-medium text-gray-700 mb-2">
							Opens At (Optional)
						</label>
						<input
							type="datetime-local"
							id="opensAt"
							bind:value={pollForm.opensAt}
							class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
							class:border-red-500={errors.opensAt}
						/>
						{#if errors.opensAt}
							<div class="mt-1 text-sm text-red-600">
								{#each errors.opensAt as error}
									<p>{error}</p>
								{/each}
							</div>
						{/if}
					</div>

					<!-- Closes At -->
					<div>
						<label for="closesAt" class="block text-sm font-medium text-gray-700 mb-2">
							Closes At (Optional)
						</label>
						<input
							type="datetime-local"
							id="closesAt"
							bind:value={pollForm.closesAt}
							class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
							class:border-red-500={errors.closesAt}
						/>
						{#if errors.closesAt}
							<div class="mt-1 text-sm text-red-600">
								{#each errors.closesAt as error}
									<p>{error}</p>
								{/each}
							</div>
						{/if}
					</div>
				</div>

				<!-- Checkboxes -->
				<div class="mt-6 space-y-4">
					<div class="flex items-center">
						<input
							type="checkbox"
							id="isPublic"
							bind:checked={pollForm.isPublic}
							class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
						/>
						<label for="isPublic" class="ml-2 block text-sm text-gray-700">
							Make this poll public (visible to everyone)
						</label>
					</div>

					<div class="flex items-center">
						<input
							type="checkbox"
							id="registrationRequired"
							bind:checked={pollForm.registrationRequired}
							class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
						/>
						<label for="registrationRequired" class="ml-2 block text-sm text-gray-700">
							Require voter registration
						</label>
					</div>
				</div>
			</div>

			<!-- Candidates -->
			<div class="bg-white shadow rounded-lg p-6">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-lg font-medium text-gray-900">Candidates</h2>
					<button
						type="button"
						onclick={addCandidate}
						class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-blue-700 bg-blue-100 hover:bg-blue-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
					>
						+ Add Candidate
					</button>
				</div>

				{#if errors.candidates}
					<div class="mb-4 text-sm text-red-600">
						{#each errors.candidates as error}
							<p>{error}</p>
						{/each}
					</div>
				{/if}

				<div class="space-y-4">
					{#each pollForm.candidates as candidate, index}
						<div class="border border-gray-200 rounded-lg p-4">
							<div class="flex items-start justify-between mb-3">
								<h3 class="text-sm font-medium text-gray-900">Candidate {index + 1}</h3>
								{#if pollForm.candidates.length > 2}
									<button
										type="button"
										onclick={() => removeCandidate(index)}
										class="text-red-600 hover:text-red-800 text-sm"
									>
										Remove
									</button>
								{/if}
							</div>

							<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
								<!-- Candidate Name -->
								<div>
									<label for="candidate-{index}-name" class="block text-sm font-medium text-gray-700 mb-1">
										Name *
									</label>
									<input
										type="text"
										id="candidate-{index}-name"
										value={candidate.name}
										oninput={(e) => updateCandidate(index, 'name', e.target.value)}
										class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
										placeholder="Candidate name"
										required
									/>
								</div>

								<!-- Candidate Description -->
								<div>
									<label for="candidate-{index}-description" class="block text-sm font-medium text-gray-700 mb-1">
										Description (Optional)
									</label>
									<input
										type="text"
										id="candidate-{index}-description"
										value={candidate.description}
										oninput={(e) => updateCandidate(index, 'description', e.target.value)}
										class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
										placeholder="Brief description"
									/>
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- Submit Error -->
			{#if submitError}
				<div class="bg-red-50 border border-red-200 rounded-md p-4">
					<h3 class="text-red-800 font-semibold">Error Updating Poll</h3>
					<p class="text-red-600">{submitError}</p>
				</div>
			{/if}

			<!-- Form Actions -->
			<div class="flex items-center justify-end space-x-4 pt-6 border-t border-gray-200">
				<button
					type="button"
					onclick={handleCancel}
					class="px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
				>
					Cancel
				</button>
				<button
					type="submit"
					disabled={isSubmitting}
					class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					{isSubmitting ? 'Updating...' : 'Update Poll'}
				</button>
			</div>
		</form>
	{/if}
</div>