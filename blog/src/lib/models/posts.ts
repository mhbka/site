export type PostStatus = 'draft' | 'published';

export interface PostSummary {
	id: string;
	title: string;
	slug: string;
	thumbnailUrl: string | null;
}

export interface Post extends PostSummary {
	authorId: string;
	contentMd: string;
	status: PostStatus;
	publishedAt: string | null;
	createdAt: string;
	updatedAt: string;
}

export interface CreatePostInput {
	title: string;
	contentMd: string;
}

export interface UpdatePostInput {
	title?: string;
	contentMd?: string;
	thumbnailUrl?: string | null;
}
