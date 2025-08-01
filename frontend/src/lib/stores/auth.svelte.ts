import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { apiClient, APIError } from '../api/client.js';
import type {
	AuthState,
	AuthTokens,
	LoginRequest,
	RegisterRequest,
	User
} from '../types.js';

// Storage keys
const AUTH_STORAGE_KEY = 'rankchoice_auth';
const TOKEN_STORAGE_KEY = 'rankchoice_tokens';

class AuthStore {
	// Svelte 5 reactive state using $state rune
	user = $state<User | null>(null);
	tokens = $state<AuthTokens | null>(null);
	isLoading = $state(false);
	error = $state<string | null>(null);

	constructor() {
		// Initialize from localStorage if in browser
		if (browser) {
			this.loadFromStorage();
		}
	}

	// Computed properties using $derived
	get isAuthenticated(): boolean {
		return this.user !== null && this.tokens !== null;
	}

	get authState(): AuthState {
		return {
			user: this.user,
			tokens: this.tokens,
			isAuthenticated: this.isAuthenticated,
			isLoading: this.isLoading,
			error: this.error
		};
	}

	// Private methods
	private loadFromStorage() {
		try {
			const savedUser = localStorage.getItem(AUTH_STORAGE_KEY);
			const savedTokens = localStorage.getItem(TOKEN_STORAGE_KEY);

			if (savedUser && savedTokens) {
				this.user = JSON.parse(savedUser);
				this.tokens = JSON.parse(savedTokens);

				// Set token in API client
				if (this.tokens?.token) {
					apiClient.setAuthToken(this.tokens.token);
				}

				// Validate token freshness and refresh if needed
				this.validateAndRefreshToken();
			}
		} catch (error) {
			console.error('Error loading auth from storage:', error);
			this.clearStorage();
		}
	}

	private saveToStorage() {
		if (!browser) return;

		try {
			if (this.user && this.tokens) {
				localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(this.user));
				localStorage.setItem(TOKEN_STORAGE_KEY, JSON.stringify(this.tokens));
			} else {
				this.clearStorage();
			}
		} catch (error) {
			console.error('Error saving auth to storage:', error);
		}
	}

	private clearStorage() {
		if (!browser) return;
		
		localStorage.removeItem(AUTH_STORAGE_KEY);
		localStorage.removeItem(TOKEN_STORAGE_KEY);
	}

	private setAuthData(user: User, tokens: AuthTokens) {
		this.user = user;
		this.tokens = tokens;
		this.error = null;

		// Set token in API client
		apiClient.setAuthToken(tokens.token);

		// Save to localStorage
		this.saveToStorage();
	}

	private async validateAndRefreshToken() {
		if (!this.tokens?.refreshToken) return;

		try {
			// Try to refresh the token
			const response = await apiClient.refreshToken({
				refreshToken: this.tokens.refreshToken
			});

			// Update with new tokens
			this.setAuthData(response.user, {
				token: response.token,
				refreshToken: response.refreshToken
			});
		} catch (error) {
			console.error('Token refresh failed:', error);
			// If refresh fails, logout user
			this.logout();
		}
	}

	// Public methods
	async login(credentials: LoginRequest): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const response = await apiClient.login(credentials);
			
			this.setAuthData(response.user, {
				token: response.token,
				refreshToken: response.refreshToken
			});

			// Small delay to ensure auth state is fully propagated before navigation
			await new Promise(resolve => setTimeout(resolve, 50));

			// Redirect to dashboard after successful login with robust navigation
			await this.navigateToDestination('/dashboard');
		} catch (error) {
			if (error instanceof APIError) {
				this.error = error.message;
			} else {
				this.error = 'Login failed. Please try again.';
			}
			throw error;
		} finally {
			this.isLoading = false;
		}
	}

	async register(userData: RegisterRequest): Promise<void> {
		this.isLoading = true;
		this.error = null;

		try {
			const response = await apiClient.register(userData);
			
			this.setAuthData(response.user, {
				token: response.token,
				refreshToken: response.refreshToken
			});

			// Small delay to ensure auth state is fully propagated before navigation
			await new Promise(resolve => setTimeout(resolve, 50));

			// Redirect to dashboard after successful registration with robust navigation
			await this.navigateToDestination('/dashboard');
		} catch (error) {
			if (error instanceof APIError) {
				this.error = error.message;
			} else {
				this.error = 'Registration failed. Please try again.';
			}
			throw error;
		} finally {
			this.isLoading = false;
		}
	}

	async logout(): Promise<void> {
		// Clear state
		this.user = null;
		this.tokens = null;
		this.error = null;

		// Clear API client token
		apiClient.setAuthToken(null);

		// Clear storage
		this.clearStorage();

		// Redirect to home page with robust navigation
		await this.navigateToDestination('/');
	}

	async refreshTokens(): Promise<void> {
		if (!this.tokens?.refreshToken) {
			throw new Error('No refresh token available');
		}

		try {
			const response = await apiClient.refreshToken({
				refreshToken: this.tokens.refreshToken
			});

			this.setAuthData(response.user, {
				token: response.token,
				refreshToken: response.refreshToken
			});
		} catch (error) {
			// If refresh fails, logout user
			await this.logout();
			throw error;
		}
	}

	// Method to handle API errors globally
	async handleAPIError(error: APIError): Promise<void> {
		if (error.code === 'UNAUTHORIZED' || error.status === 401) {
			// Token is invalid, try to refresh
			try {
				await this.refreshTokens();
			} catch (refreshError) {
				// Refresh failed, logout user
				await this.logout();
			}
		}
	}

	// Clear any error messages
	clearError(): void {
		this.error = null;
	}

	// Robust navigation method for high-load parallel execution
	private async navigateToDestination(destination: string, maxRetries: number = 5): Promise<void> {
		let retries = 0;
		
		while (retries < maxRetries) {
			try {
				// Progressive delay to prevent race conditions in parallel execution
				if (retries > 0) {
					const delay = Math.min(200 + (retries * 100), 500); // Cap at 500ms
					await new Promise(resolve => setTimeout(resolve, delay));
				}
				
				// Attempt navigation with explicit options
				await goto(destination, { 
					replaceState: false,
					keepFocus: false,
					invalidateAll: true
				});
				
				// Give navigation time to complete
				if (browser) {
					await new Promise(resolve => setTimeout(resolve, 100)); // Wait for navigation
				}
				
				return; // Success
			} catch (error) {
				retries++;
				console.warn(`Navigation attempt ${retries}/${maxRetries} to ${destination} failed:`, error);
				
				if (retries >= maxRetries) {
					// Final attempt failed - log but don't block auth flow
					console.error(`Failed to navigate to ${destination} after ${maxRetries} attempts:`, error);
					// Auth operation succeeded, navigation failed - tests will handle this
					return;
				}
			}
		}
	}

	// Check if user has specific role
	hasRole(role: string): boolean {
		return this.user?.role === role;
	}

	// Check if user is admin
	get isAdmin(): boolean {
		return this.hasRole('admin');
	}

	// Get user's display name
	get displayName(): string {
		return this.user?.name || this.user?.email || 'User';
	}
}

// Export singleton instance
export const authStore = new AuthStore(); 