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
	hasVoted: boolean;
	votedAt?: string;
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

export interface RCVRound {
	roundNumber: number;
	voteCounts: Record<string, number>;
	eliminated?: string;
	winner?: string;
	exhaustedBallots: number;
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

// DnD types for svelte-dnd-action
export interface DndEvent<T = any> {
	items: T[];
	info: {
		source: string;
		trigger: string;
	};
} 