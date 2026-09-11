import type { PostStatus } from './models/posts.ts';

/** Returns the editor labels that match a post's publication state. */
export function getPostEditorLabels(status: PostStatus | undefined) {
	if (status === 'published') {
		return { state: '[published]', submit: 'save changes' };
	}

	return { state: '[draft]', submit: 'save draft' };
}
