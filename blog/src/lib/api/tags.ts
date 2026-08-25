import type { ApiRequest } from '../models/api.ts';
import type { TagSummary } from '../models/tags.ts';

export function createTagsApi(request: ApiRequest) {
	return {
		listTags: () => request<TagSummary[]>('/tags'),
	};
}
