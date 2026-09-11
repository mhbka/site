import type { ApiRequest } from '../models/api.ts';
import type { TagSummary } from '../models/tags.ts';

/** Creates requests for listing post tags. */
export function createTagsApi(request: ApiRequest) {
	return {
		listTags: () => request<TagSummary[]>('/tags'),
	};
}
