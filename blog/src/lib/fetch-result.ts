export type FetchResult<T> =
	| { data: T; error: undefined }
	| { data: undefined; error: string };

/** Converts a request result into data or a user-facing error message. */
export async function fetchResult<T>(request: () => Promise<T>, error: string): Promise<FetchResult<T>> {
	try {
		return { data: await request(), error: undefined };
	} catch {
		return { data: undefined, error };
	}
}
