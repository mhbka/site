/** Identifies whether a post is private or publicly available. */
export type PostStatus = 'draft' | 'published';

/** Represents the fields shown for a published post listing. */
export interface PostSummary {
	id: string;
	title: string;
	slug: string;
	thumbnailUrl: string | null;
	tags: string[];
	publishedAt: string;
}

/** Represents the fields shown for a draft post listing. */
export interface DraftPostSummary {
	id: string;
	title: string;
	slug: string;
	thumbnailUrl: string | null;
	tags: string[];
	updatedAt: string;
}

/** Represents the full editable post returned by the backend. */
export interface Post extends Omit<PostSummary, 'publishedAt'> {
	authorId: string;
	contentMd: string;
	status: PostStatus;
	publishedAt: string | null;
	tags: string[];
	createdAt: string;
	updatedAt: string;
}

/** Supplies fields for creating a post. */
export interface CreatePostInput {
	title: string;
	contentMd: string;
	slug?: string;
	tags?: string[];
}

/** Supplies the mutable fields for a post update. */
export interface UpdatePostInput {
	title?: string;
	contentMd?: string;
	thumbnailUrl?: string | null;
	slug?: string;
	tags?: string[];
}
