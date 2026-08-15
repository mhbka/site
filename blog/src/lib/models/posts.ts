export type PostStatus = 'draft' | 'published';

export interface PostSummary {
	id: string;
	title: string;
	slug: string;
	thumbnailUrl: string | null;
	publishedAt: string;
}

export interface DraftPostSummary {
	id: string;
	title: string;
	slug: string;
	thumbnailUrl: string | null;
	updatedAt: string;
}

export interface Post extends Omit<PostSummary, 'publishedAt'> {
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
	slug?: string;
}

export interface UpdatePostInput {
	title?: string;
	contentMd?: string;
	thumbnailUrl?: string | null;
	slug?: string;
}
