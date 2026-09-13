import { createApiClient, DEFAULT_API_BASE_URL } from './api/client.ts';
import { createCommentsApi } from './api/comments.ts';
import { createPixApi } from './api/pix.ts';
import { createPostsApi } from './api/posts.ts';
import { createTagsApi } from './api/tags.ts';
import type { BlogApiOptions } from './models/api.ts';

export { ApiError, DEFAULT_API_BASE_URL } from './api/client.ts';
export type { BlogApiOptions } from './models/api.ts';
export type { Comment, CreateCommentInput, UpdateCommentInput } from './models/comments.ts';
export type { CreatePostInput, DraftPostSummary, Post, PostStatus, PostSummary, UpdatePostInput } from './models/posts.ts';
export type { TagSummary } from './models/tags.ts';
export type { Pix, PixPage, PixTagUpdate, PixUpload } from './models/pix.ts';

/** Combines each backend resource client into the application API. */
export function createBlogApi(options: BlogApiOptions = {}) {
	const request = createApiClient(options);
	return {
		...createPostsApi(request),
		...createTagsApi(request),
		...createCommentsApi(request),
		...createPixApi(request, options.fetch),
	};
}

/** Provides the application's default backend API client. */
export const blogApi = createBlogApi({ baseUrl: DEFAULT_API_BASE_URL });
