export interface ImageUploadResponse {
	url: string;
}

export async function uploadImage(
	file: File,
	uploadUrl: string,
	token?: string,
	fetcher: typeof fetch = fetch,
): Promise<string> {
	const body = new FormData();
	body.append('file', file);

	const response = await fetcher(uploadUrl, {
		method: 'POST',
		body,
		headers: token ? { Authorization: `Bearer ${token}` } : undefined,
	});

	if (!response.ok) {
		throw new Error('Image upload failed. Please try again.');
	}

	const data: unknown = await response.json();
	if (!isImageUploadResponse(data)) {
		throw new Error('Image upload did not return an image URL.');
	}

	return data.url;
}

function isImageUploadResponse(data: unknown): data is ImageUploadResponse {
	return typeof data === 'object' && data !== null && 'url' in data && typeof data.url === 'string';
}
