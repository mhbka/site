export interface Comment {
	id: string;
	postId: string;
	authorId: string;
	parentCommentId: string | null;
	body: string;
	status: string;
	createdAt: string;
	updatedAt: string;
	deletedAt: string | null;
}

export interface UpdateCommentInput {
	body: string;
}

export interface CreateCommentInput {
	body: string;
	parentCommentId?: string | null;
}
