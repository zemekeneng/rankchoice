// User types
export interface User {
	id: string;
	email: string;
	name?: string;
	role: 'pollster' | 'admin';
	createdAt: string;
	updatedAt: string;
}

// Poll types
export interface Poll {
	id: string;
	userId: string;
	title: string;
	description?: string;
	pollType: 'single_winner' | 'multi_winner';
	numWinners: number;
	opensAt?: string;
	closesAt?: string;
	isPublic: boolean;
	registrationRequired: boolean;
	createdAt: string;
	updatedAt: string;
	candidates?: Candidate[];
	candidateCount?: number; // From list view
	voteCount?: number; // From list view
}

// Candidate types
export interface Candidate {
	id: string;
	pollId?: string;
	name: string;
	description?: string;
	displayOrder: number;
	rank?: number; // Added during voting
}

// Voting types
export interface Ballot {
	id: string;
	voterId: string;
	pollId: string;
	submittedAt: string;
	rankings: Ranking[];
}

export interface Ranking {
	candidateId: string;
	rank: number;
}

export interface Voter {
	id: string;
	pollId: string;
	email?: string;
	ballotToken: string;
	hasVoted: boolean;
	invitedAt: string;
	votedAt?: string;
	votingUrl: string;
}

export interface CreateVoterRequest {
	email?: string;
}

export interface VotersListResponse {
	voters: Voter[];
	total: number;
	votedCount: number;
	pendingCount: number;
}

// Results types
export interface PollResults {
	pollId: string;
	totalVotes: number;
	status: 'in_progress' | 'completed';
	winner?: WinnerInfo;
	finalRankings: FinalRanking[];
	rounds?: RCVRound[];
}

export interface WinnerInfo {
	candidateId: string;
	name: string;
	finalVotes: number;
	percentage: number;
}

export interface FinalRanking {
	position: number;
	candidateId: string;
	name: string;
	votes: number;
	percentage: number;
	eliminatedRound?: number;
}

export interface VoteCounts {
	votes: number;
	percentage: number;
}

export interface EliminatedCandidate {
	candidate_id: string;
	name: string;
	votes: number;
}

export interface WinnerCandidate {
	candidate_id: string;
	name: string;
	votes: number;
}

export interface RCVRound {
	roundNumber: number;
	vote_counts: Record<string, VoteCounts>;
	eliminated?: string | EliminatedCandidate;
	winner?: string | WinnerCandidate;
	exhausted_ballots: number;
	tiebreak_reason?: string;
}

// API types
export interface ApiResponse<T> {
	success: boolean;
	data: T | null;
	error: ApiError | null;
	metadata: {
		timestamp: string;
		version: string;
		requestId?: string;
	};
}

export interface ApiError {
	code: string;
	message: string;
	details?: any;
}

// Form types
export interface CreatePollForm {
	title: string;
	description: string;
	pollType: 'single_winner' | 'multi_winner';
	numWinners: number;
	opensAt?: string;
	closesAt?: string;
	isPublic: boolean;
	registrationRequired: boolean;
	candidates: CreateCandidateForm[];
}

export interface CreateCandidateForm {
	name: string;
	description?: string;
}

// Authentication types
export interface AuthTokens {
	token: string;
	refreshToken: string;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
	name?: string;
}

export interface AuthResponse {
	user: User;
	token: string;
	refreshToken: string;
}

export interface RefreshTokenRequest {
	refreshToken: string;
}

// Auth store state
export interface AuthState {
	user: User | null;
	tokens: AuthTokens | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
}

// Form validation types
export interface FormErrors {
	[key: string]: string[];
}

export interface LoginForm {
	email: string;
	password: string;
}

export interface RegisterForm {
	email: string;
	password: string;
	confirmPassword: string;
	name: string;
}

// DnD types for svelte-dnd-action
export interface DndEvent<T = any> {
	items: T[];
	info: {
		source: string;
		trigger: string;
	};
} 