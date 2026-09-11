import type { PostStatus } from './models/posts.ts';

/** Returns the editor labels that match a post's publication state. */
export function getPostEditorLabels(status: PostStatus | undefined) {
	if (status === 'published') {
		return { state: '[Published]', submit: 'Save changes' };
	}

	return { state: '[Draft]', submit: 'Save draft' };
}
