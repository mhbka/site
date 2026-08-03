export interface BlogApiOptions {
	baseUrl?: string;
	fetch?: typeof globalThis.fetch;
}

export type ApiRequest = <T>(path: string, options?: RequestInit, token?: string) => Promise<T>;
