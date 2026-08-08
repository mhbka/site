import type { ApiRequest } from '../models/api.ts';
import type { CreatePostInput, Post, PostSummary, UpdatePostInput } from '../models/posts.ts';

export function createPostsApi(request: ApiRequest) {
	return {
		listPosts: (page = 1, size = 50) =>
			request<PostSummary[]>(`/posts?page=${encodeURIComponent(page)}&size=${encodeURIComponent(size)}`),
		getPost: (slug: string) => request<Post>(`/posts/${encodeURIComponent(slug)}`),
		createPost: (input: CreatePostInput, token: string) =>
			request<Post>('/posts', { method: 'POST', body: JSON.stringify(input) }, token),
		updatePost: (id: string, input: UpdatePostInput, token: string) =>
			request<Post>(`/posts/id/${encodeURIComponent(id)}`, { method: 'PUT', body: JSON.stringify(input) }, token),
		publishPost: (id: string, token: string) =>
			request<Post>(`/posts/id/${encodeURIComponent(id)}/publish`, { method: 'POST' }, token),
		deletePost: (id: string, token: string) =>
			request<void>(`/posts/id/${encodeURIComponent(id)}`, { method: 'DELETE' }, token),
	};
}
