import type {
	ApiResponse,
	AuthResponse,
	LoginRequest,
	RegisterRequest,
	RefreshTokenRequest,
	User,
	Poll,
	CreatePollForm,
	Candidate,
	PollResults,
	RCVRound,
	Voter,
	CreateVoterRequest,
	VotersListResponse
} from '../types.js';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8081/api';

class APIError extends Error {
	constructor(
		message: string,
		public code: string,
		public status: number,
		public details?: any
	) {
		super(message);
		this.name = 'APIError';
	}
}

class APIClient {
	private baseURL: string;
	private authToken: string | null = null;

	constructor(baseURL: string = API_BASE_URL) {
		this.baseURL = baseURL;
	}

	setAuthToken(token: string | null) {
		this.authToken = token;
	}

	private async request<T>(
		endpoint: string,
		options: RequestInit = {}
	): Promise<ApiResponse<T>> {
		const url = `${this.baseURL}${endpoint}`;
		
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			...((options.headers as Record<string, string>) || {})
		};

		if (this.authToken) {
			headers.Authorization = `Bearer ${this.authToken}`;
		}

		const config: RequestInit = {
			...options,
			headers
		};

		try {
			const response = await fetch(url, config);
			const data: ApiResponse<T> = await response.json();

			if (!data.success) {
				throw new APIError(
					data.error?.message || 'API request failed',
					data.error?.code || 'UNKNOWN_ERROR',
					response.status,
					data.error?.details
				);
			}

			return data;
		} catch (error) {
			if (error instanceof APIError) {
				throw error;
			}

			// Network or parsing error
			throw new APIError(
				'Network error or invalid response',
				'NETWORK_ERROR',
				0,
				error
			);
		}
	}

	// Authentication endpoints
	async login(credentials: LoginRequest): Promise<AuthResponse> {
		const response = await this.request<AuthResponse>('/auth/login', {
			method: 'POST',
			body: JSON.stringify(credentials)
		});
		return response.data!;
	}

	async register(userData: RegisterRequest): Promise<AuthResponse> {
		const response = await this.request<AuthResponse>('/auth/register', {
			method: 'POST',
			body: JSON.stringify(userData)
		});
		return response.data!;
	}

	async refreshToken(refreshData: RefreshTokenRequest): Promise<AuthResponse> {
		const response = await this.request<AuthResponse>('/auth/refresh', {
			method: 'POST',
			body: JSON.stringify(refreshData)
		});
		return response.data!;
	}

	// Poll endpoints
	async getPolls(params?: {
		page?: number;
		limit?: number;
		status?: string;
		sort?: string;
		order?: string;
	}): Promise<{ polls: Poll[]; total: number; totalPages: number }> {
		const searchParams = new URLSearchParams();
		if (params) {
			Object.entries(params).forEach(([key, value]) => {
				if (value !== undefined) {
					searchParams.append(key, value.toString());
				}
			});
		}

		const endpoint = `/polls${searchParams.toString() ? `?${searchParams.toString()}` : ''}`;
		const response = await this.request<{ items: Poll[]; total: number; total_pages: number }>(endpoint);
		
		// Transform backend response format to frontend expected format
		return {
			polls: response.data!.items.map((poll: any) => ({
				...this.mapPollFromApi(poll),
				candidateCount: poll.candidate_count,
				voteCount: poll.vote_count
			})),
			total: response.data!.total,
			totalPages: response.data!.total_pages
		};
	}

	async getPoll(id: string): Promise<Poll> {
		const response = await this.request<any>(`/polls/${id}`);
		
		return this.mapPollFromApi(response.data!);
	}

	async getPublicPoll(id: string): Promise<Poll> {
		const response = await this.request<any>(`/public/polls/${id}`);
		
		return this.mapPollFromApi(response.data!);
	}

	async submitAnonymousVote(pollId: string, rankings: Array<{ candidateId: string; rank: number }>): Promise<{
		ballot: {
			id: string;
			submitted_at: string;
		};
		receipt: {
			receipt_code: string;
			verification_url: string;
		};
	}> {
		const requestBody = {
			rankings: rankings.map(r => ({
				candidate_id: r.candidateId,
				rank: r.rank
			}))
		};

		const response = await this.request<{
			ballot: {
				id: string;
				submitted_at: string;
			};
			receipt: {
				receipt_code: string;
				verification_url: string;
			};
		}>(`/public/polls/${pollId}/vote`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(requestBody),
		});
		
		return response.data!;
	}

	private mapPollFromApi(poll: any): Poll {
		return {
			id: poll.id,
			userId: poll.user_id,
			title: poll.title,
			description: poll.description,
			pollType: poll.poll_type,
			numWinners: poll.num_winners,
			opensAt: poll.opens_at,
			closesAt: poll.closes_at,
			isPublic: poll.is_public,
			registrationRequired: poll.registration_required,
			createdAt: poll.created_at,
			updatedAt: poll.updated_at,
			candidates: poll.candidates?.map((candidate: any) => ({
				id: candidate.id,
				pollId: candidate.poll_id,
				name: candidate.name,
				description: candidate.description,
				displayOrder: candidate.display_order,
				createdAt: candidate.created_at
			}))
		};
	}

	async createPoll(pollData: CreatePollForm): Promise<Poll> {
		// Map camelCase frontend fields to snake_case backend fields
		// Helper function to convert datetime-local format to ISO format
		const formatDateTime = (dateTimeStr: string): string | null => {
			if (!dateTimeStr || dateTimeStr.trim() === '') return null;
			// Convert "2025-08-04T17:36" to "2025-08-04T17:36:00Z"
			return new Date(dateTimeStr).toISOString();
		};

		const backendPollData = {
			title: pollData.title,
			description: pollData.description,
			poll_type: pollData.pollType, // camelCase -> snake_case
			num_winners: pollData.numWinners, // camelCase -> snake_case
			opens_at: formatDateTime(pollData.opensAt),
			closes_at: formatDateTime(pollData.closesAt),
			is_public: pollData.isPublic, // camelCase -> snake_case
			registration_required: pollData.registrationRequired, // camelCase -> snake_case
			candidates: pollData.candidates
		};

		const response = await this.request<any>('/polls', {
			method: 'POST',
			body: JSON.stringify(backendPollData)
		});
		
		return this.mapPollFromApi(response.data!);
	}

	async updatePoll(id: string, pollData: Partial<CreatePollForm>): Promise<Poll> {
		const response = await this.request<Poll>(`/polls/${id}`, {
			method: 'PUT',
			body: JSON.stringify(pollData)
		});
		return response.data!;
	}

	async deletePoll(id: string): Promise<void> {
		await this.request(`/polls/${id}`, {
			method: 'DELETE'
		});
	}

	// Candidate endpoints
	async getCandidates(pollId: string): Promise<Candidate[]> {
		const response = await this.request<Candidate[]>(`/polls/${pollId}/candidates`);
		return response.data!;
	}

	async addCandidate(pollId: string, candidateData: { name: string; description?: string }): Promise<Candidate> {
		const response = await this.request<Candidate>(`/polls/${pollId}/candidates`, {
			method: 'POST',
			body: JSON.stringify(candidateData)
		});
		return response.data!;
	}

	async updateCandidate(id: string, candidateData: { name: string; description?: string }): Promise<Candidate> {
		const response = await this.request<Candidate>(`/candidates/${id}`, {
			method: 'PUT',
			body: JSON.stringify(candidateData)
		});
		return response.data!;
	}

	async deleteCandidate(id: string): Promise<void> {
		await this.request(`/candidates/${id}`, {
			method: 'DELETE'
		});
	}

	async reorderCandidates(pollId: string, candidateOrder: string[]): Promise<void> {
		await this.request(`/polls/${pollId}/candidates/order`, {
			method: 'PUT',
			body: JSON.stringify({ candidate_order: candidateOrder })
		});
	}

	// Voting endpoints (public - no auth required)
	async getBallot(token: string): Promise<{
		poll: Poll;
		voter: { id: string; has_voted: boolean };
	}> {
		// Temporarily remove auth token for public voting endpoints
		const originalToken = this.authToken;
		this.setAuthToken(null);
		
		try {
			const response = await this.request<{
				poll: any; // Raw API response
				voter: { id: string; has_voted: boolean };
			}>(`/vote/${token}`);
			
			// Map the poll data to match frontend types
			const mappedData = {
				poll: this.mapPollFromApi(response.data!.poll),
				voter: response.data!.voter
			};
			
			return mappedData;
		} finally {
			this.setAuthToken(originalToken);
		}
	}

	async submitBallot(token: string, rankings: { candidate_id: string; rank: number }[]): Promise<{
		ballot: { id: string; submitted_at: string };
		receipt: { receipt_code: string; verification_url: string };
	}> {
		// Temporarily remove auth token for public voting endpoints
		const originalToken = this.authToken;
		this.setAuthToken(null);
		
		try {
			const response = await this.request<{
				ballot: { id: string; submitted_at: string };
				receipt: { receipt_code: string; verification_url: string };
			}>(`/vote/${token}`, {
				method: 'POST',
				body: JSON.stringify({ rankings })
			});
			return response.data!;
		} finally {
			this.setAuthToken(originalToken);
		}
	}

	async getVotingReceipt(token: string): Promise<{
		ballot_id: string;
		submitted_at: string;
		poll_id: string;
		receipt_code: string;
		verification_url: string;
	}> {
		// Temporarily remove auth token for public voting endpoints
		const originalToken = this.authToken;
		this.setAuthToken(null);
		
		try {
			const response = await this.request<{
				ballot_id: string;
				submitted_at: string;
				poll_id: string;
				receipt_code: string;
				verification_url: string;
			}>(`/vote/${token}/receipt`);
			return response.data!;
		} finally {
			this.setAuthToken(originalToken);
		}
	}

	// Results endpoints
	async getPollResults(pollId: string): Promise<PollResults> {
		const response = await this.request<PollResults>(`/polls/${pollId}/results`);
		return response.data!;
	}

	async getRCVRounds(pollId: string): Promise<{
		rounds: RCVRound[];
		total_ballots: number;
		exhausted_ballots: number;
	}> {
		const response = await this.request<{
			rounds: RCVRound[];
			total_ballots: number;
			exhausted_ballots: number;
		}>(`/polls/${pollId}/results/rounds`);
		return response.data!;
	}

	async getAnonymousBallots(pollId: string): Promise<{
		poll_id: string;
		poll_title: string;
		total_ballots: number;
		ballots: Array<{
			ballot_id: string;
			submitted_at: string;
			rankings: Array<{
				candidate_name: string;
				rank: number;
			}>;
		}>;
	}> {
		const response = await this.request<{
			poll_id: string;
			poll_title: string;
			total_ballots: number;
			ballots: Array<{
				ballot_id: string;
				submitted_at: string;
				rankings: Array<{
					candidate_name: string;
					rank: number;
				}>;
			}>;
		}>(`/polls/${pollId}/ballots/anonymous`);
		return response.data!;
	}

	// Voter management endpoints
	async createVoter(pollId: string, voterData: CreateVoterRequest): Promise<Voter> {
		const response = await this.request<Voter>(`/polls/${pollId}/invite`, {
			method: 'POST',
			body: JSON.stringify(voterData)
		});
		return response.data!;
	}

	async getVoters(pollId: string): Promise<VotersListResponse> {
		const response = await this.request<VotersListResponse>(`/polls/${pollId}/voters`);
		return response.data!;
	}
}

// Export singleton instance
export const apiClient = new APIClient();
export { APIError };
export type { APIClient }; 