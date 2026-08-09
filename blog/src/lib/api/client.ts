import type { ApiRequest, BlogApiOptions } from '../models/api.ts';

export class ApiError extends Error {
	public readonly status: number;
	public readonly body: unknown;

	constructor(status: number, body: unknown) {
		super(`API request failed with status ${status}`);
		this.name = 'ApiError';
		this.status = status;
		this.body = body;
	}
}

const environment = (import.meta as ImportMeta & { env?: Record<string, string | undefined> }).env;
export const DEFAULT_API_BASE_URL = environment?.BACKEND_URL ?? 'http://localhost:8080';

export function createApiClient({
	baseUrl = DEFAULT_API_BASE_URL,
	fetch: fetcher = globalThis.fetch,
}: BlogApiOptions = {}): ApiRequest {
	return async function request<T>(
		path: string,
		options: RequestInit = {},
		token?: string,
	): Promise<T> {
		const headers = new Headers(options.headers);
		if (options.body) headers.set('Content-Type', 'application/json');
		if (token) headers.set('Authorization', `Bearer ${token}`);

		const response = await fetcher(`${baseUrl.replace(/\/$/, '')}${path}`, {
			...options,
			headers,
		});

		if (!response.ok) {
			const body = await response.json().catch(() => undefined);
			throw new ApiError(response.status, body);
		}

		if (response.status === 204) return undefined as T;
		return response.json() as Promise<T>;
	};
}
