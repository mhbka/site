import type { ApiRequest } from '../models/api.ts';
import type { Comment, CreateCommentInput } from '../models/comments.ts';

export function createCommentsApi(request: ApiRequest) {
	return {
		listComments: (postId: string) =>
			request<Comment[]>(`/posts/id/${encodeURIComponent(postId)}/comments`),
		createComment: (postId: string, input: CreateCommentInput, token: string) =>
			request<Comment>(
				`/posts/id/${encodeURIComponent(postId)}/comments`,
				{ method: 'POST', body: JSON.stringify(input) },
				token,
			),
	};
}
