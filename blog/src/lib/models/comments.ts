/** Represents a post comment returned by the backend. */
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

/** Supplies the mutable fields for a comment update. */
export interface UpdateCommentInput {
	body: string;
}

/** Supplies the content and optional parent for a new comment. */
export interface CreateCommentInput {
	body: string;
	parentCommentId?: string | null;
}
