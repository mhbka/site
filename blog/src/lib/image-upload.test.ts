import assert from 'node:assert/strict';
import test from 'node:test';

import { uploadImage } from './image-upload.ts';

test('uploads an image and returns the URL supplied by the upload service', async () => {
	let request: RequestInit | undefined;
	const fetcher: typeof fetch = async (_input, init) => {
		request = init;
		return Response.json({ url: 'https://images.example.test/post.png' });
	};

	const url = await uploadImage(
		new File(['image'], 'post.png', { type: 'image/png' }),
		'https://uploads.example.test/images',
		'access-token',
		fetcher,
	);

	assert.equal(url, 'https://images.example.test/post.png');
	assert.equal(request?.method, 'POST');
	assert.equal(new Headers(request?.headers).get('Authorization'), 'Bearer access-token');
	assert.ok(request?.body instanceof FormData);
});

test('rejects upload responses without an image URL', async () => {
	const fetcher: typeof fetch = async () => Response.json({});

	await assert.rejects(
		uploadImage(
			new File(['image'], 'post.png', { type: 'image/png' }),
			'https://uploads.example.test/images',
			undefined,
			fetcher,
		),
		/image URL/,
	);
});
