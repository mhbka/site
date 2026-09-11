/** Configures the base URL and fetch implementation for a blog API client. */
export interface BlogApiOptions {
	baseUrl?: string;
	fetch?: typeof globalThis.fetch;
}

/** Represents an authenticated request made through the backend client. */
export type ApiRequest = <T>(path: string, options?: RequestInit, token?: string) => Promise<T>;
