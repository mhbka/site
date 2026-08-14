import { createApiClient, DEFAULT_API_BASE_URL } from './api/client.ts';
import { createCommentsApi } from './api/comments.ts';
import { createPostsApi } from './api/posts.ts';
import type { BlogApiOptions } from './models/api.ts';

export { ApiError, DEFAULT_API_BASE_URL } from './api/client.ts';
export type { BlogApiOptions } from './models/api.ts';
export type { Comment, CreateCommentInput, UpdateCommentInput } from './models/comments.ts';
export type { CreatePostInput, DraftPostSummary, Post, PostStatus, PostSummary, UpdatePostInput } from './models/posts.ts';

export function createBlogApi(options: BlogApiOptions = {}) {
	const request = createApiClient(options);
	return {
		...createPostsApi(request),
		...createCommentsApi(request),
	};
}

export const blogApi = createBlogApi({ baseUrl: DEFAULT_API_BASE_URL });
