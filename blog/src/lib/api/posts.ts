import type { ApiRequest } from '../models/api.ts';
import type { CreatePostInput, DraftPostSummary, Post, PostSummary, UpdatePostInput } from '../models/posts.ts';

export function createPostsApi(request: ApiRequest) {
	return {
		getAuthorStatus: (token: string) => request<{ isAuthor: boolean }>('/users/is-author', {}, token),
		listPosts: (page = 1, size = 50) =>
			request<PostSummary[]>(`/posts?page=${encodeURIComponent(page)}&size=${encodeURIComponent(size)}`),
		listDrafts: (token: string) => request<DraftPostSummary[]>('/posts/drafts', {}, token),
		getPost: (slug: string) => request<Post>(`/posts/${encodeURIComponent(slug)}`),
		getPostById: (id: string, token: string) =>
			request<Post>(`/posts/id/${encodeURIComponent(id)}`, {}, token),
		createPost: (input: CreatePostInput, token: string) =>
			request<Post>('/posts', { method: 'POST', body: JSON.stringify(input) }, token),
		updatePost: (id: string, input: UpdatePostInput, token: string) =>
			request<Post>(`/posts/id/${encodeURIComponent(id)}`, { method: 'PUT', body: JSON.stringify(input) }, token),
		publishPost: (id: string, token: string) =>
			request<Post>(`/posts/id/${encodeURIComponent(id)}/publish`, { method: 'POST' }, token),
		movePostToDraft: (id: string, token: string) =>
			request<Post>(`/posts/id/${encodeURIComponent(id)}/draft`, { method: 'POST' }, token),
		deletePost: (id: string, token: string) =>
			request<void>(`/posts/id/${encodeURIComponent(id)}`, { method: 'DELETE' }, token),
	};
}
