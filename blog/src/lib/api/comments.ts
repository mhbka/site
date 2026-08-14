import type { ApiRequest } from '../models/api.ts';
import type { Comment, CreateCommentInput, UpdateCommentInput } from '../models/comments.ts';

export function createCommentsApi(request: ApiRequest) {
	return {
		listComments: (postId: string) =>
			request<Comment[]>(`/comments/post/${encodeURIComponent(postId)}`),
		createComment: (postId: string, input: CreateCommentInput, token: string) =>
			request<Comment>(
				`/comments/post/${encodeURIComponent(postId)}`,
				{ method: 'POST', body: JSON.stringify(input) },
				token,
			),
		updateComment: (id: string, input: UpdateCommentInput, token: string) =>
			request<Comment>(`/comments/${encodeURIComponent(id)}`, { method: 'PUT', body: JSON.stringify(input) }, token),
		deleteComment: (id: string, token: string) =>
			request<void>(`/comments/${encodeURIComponent(id)}`, { method: 'DELETE' }, token),
	};
}
