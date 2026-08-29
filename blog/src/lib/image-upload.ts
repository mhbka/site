export interface ImageUploadRequestResponse {
	uploadUrl: string;
	publicUrl: string;
}

export async function uploadImage(
	file: File,
	mediaApiUrl: string,
	postId: string,
	token: string,
	fetcher: typeof fetch = fetch,
): Promise<string> {
	const requestResponse = await fetcher(mediaApiUrl, {
		method: 'POST',
		headers: {
		Authorization: `Bearer ${token}`,
		'Content-Type': 'application/json',
		},
		body: JSON.stringify({ postId, contentType: file.type }),
	});

	if (!requestResponse.ok) {
		throw new Error('Unable to prepare image upload. Please try again.');
	}

	const upload: unknown = await requestResponse.json();
	if (!isImageUploadRequestResponse(upload)) {
		throw new Error('Image upload did not return upload details.');
	}

	const uploadResponse = await fetcher(upload.uploadUrl, {
		method: 'PUT',
		headers: { 'Content-Type': file.type },
		body: file,
	});

	if (!uploadResponse.ok) {
		throw new Error('Image upload failed. Please try again.');
	}

	return upload.publicUrl;
}

function isImageUploadRequestResponse(data: unknown): data is ImageUploadRequestResponse {
	return typeof data === 'object'
		&& data !== null
		&& 'uploadUrl' in data
		&& typeof data.uploadUrl === 'string'
		&& 'publicUrl' in data
		&& typeof data.publicUrl === 'string';
}
