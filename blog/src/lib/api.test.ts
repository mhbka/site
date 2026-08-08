import assert from 'node:assert/strict';
import test from 'node:test';

import { ApiError, createBlogApi } from './api.ts';

function createFetch(response: Response) {
	const calls: Array<[string, RequestInit | undefined]> = [];
	return {
		calls,
		fetch: async (input: string | URL | Request, init?: RequestInit) => {
			calls.push([String(input), init]);
			return response;
		},
	};
}

test('sends authenticated write requests to the matching backend route', async () => {
	const mock = createFetch(
		Response.json({ id: 'post-1', title: 'Hello', slug: 'hello', thumbnailUrl: null }),
	);
	const api = createBlogApi({ baseUrl: 'https://api.example.test/', fetch: mock.fetch });

	await api.updatePost(
		'post/id',
		{ contentMd: 'Updated content', thumbnailUrl: 'https://example.test/image.png' },
		'access-token',
	);

	const [url, options] = mock.calls[0];
	assert.equal(url, 'https://api.example.test/posts/id/post%2Fid');
	assert.equal(options?.method, 'PUT');
	assert.equal(new Headers(options?.headers).get('Authorization'), 'Bearer access-token');
	assert.equal(new Headers(options?.headers).get('Content-Type'), 'application/json');
	assert.deepEqual(JSON.parse(String(options?.body)), {
		contentMd: 'Updated content',
		thumbnailUrl: 'https://example.test/image.png',
	});
});

test('returns no value for a successful delete', async () => {
	const mock = createFetch(new Response(null, { status: 204 }));
	const api = createBlogApi({ baseUrl: 'https://api.example.test', fetch: mock.fetch });

	assert.equal(await api.deletePost('post-1', 'access-token'), undefined);
	assert.equal(mock.calls[0][0], 'https://api.example.test/posts/id/post-1');
});

test('sends requested pagination values when listing posts', async () => {
	const mock = createFetch(Response.json([]));
	const api = createBlogApi({ baseUrl: 'https://api.example.test', fetch: mock.fetch });

	await api.listPosts(3, 20);

	assert.equal(mock.calls[0][0], 'https://api.example.test/posts?page=3&size=20');
});

test('throws an ApiError that includes the response status and body', async () => {
	const mock = createFetch(Response.json({ error: 'post not found' }, { status: 404 }));
	const api = createBlogApi({ baseUrl: 'https://api.example.test', fetch: mock.fetch });

	await assert.rejects(api.getPost('missing'), (error: unknown) => {
		assert.ok(error instanceof ApiError);
		assert.equal(error.status, 404);
		assert.deepEqual(error.body, { error: 'post not found' });
		return true;
	});
});
