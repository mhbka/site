export type FetchResult<T> =
	| { data: T; error: undefined }
	| { data: undefined; error: string };

export async function fetchResult<T>(request: () => Promise<T>, error: string): Promise<FetchResult<T>> {
	try {
		return { data: await request(), error: undefined };
	} catch {
		return { data: undefined, error };
	}
}
