import assert from 'node:assert/strict';
import test from 'node:test';

import { uploadImage } from './image-upload.ts';

test('requests a presigned URL, uploads the file, and returns the public URL', async () => {
	const requests: Array<{ input: RequestInfo | URL; init?: RequestInit }> = [];
	const fetcher: typeof fetch = async (input, init) => {
		requests.push({ input, init });
		if (requests.length === 1) {
			return Response.json({
				uploadUrl: 'https://storage.example.test/presigned-upload',
				publicUrl: 'https://images.example.test/post.png',
			});
		}
		return new Response(null, { status: 200 });
	};

	const url = await uploadImage(
		new File(['image'], 'post.png', { type: 'image/png' }),
		'https://api.example.test/media/uploads',
		'post-id',
		'access-token',
		fetcher,
	);

	assert.equal(url, 'https://images.example.test/post.png');
	assert.equal(requests[0].init?.method, 'POST');
	assert.equal(new Headers(requests[0].init?.headers).get('Authorization'), 'Bearer access-token');
	assert.equal(requests[0].init?.body, JSON.stringify({ postId: 'post-id', contentType: 'image/png' }));
	assert.equal(requests[1].input, 'https://storage.example.test/presigned-upload');
	assert.equal(requests[1].init?.method, 'PUT');
	assert.equal(new Headers(requests[1].init?.headers).get('Content-Type'), 'image/png');
});

test('rejects upload responses without an image URL', async () => {
	const fetcher: typeof fetch = async () => Response.json({});

	await assert.rejects(
		uploadImage(
			new File(['image'], 'post.png', { type: 'image/png' }),
			'https://api.example.test/media/uploads',
			'post-id',
			'access-token',
			fetcher,
		),
		/upload details/,
	);
});
