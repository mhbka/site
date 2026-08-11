export function normalizeTag(value: string): string {
	return value.toLowerCase().replace(/\s+/g, '');
}

export function addTag(tags: readonly string[], value: string): string[] {
	const tag = normalizeTag(value);
	return tag && !tags.includes(tag) ? [...tags, tag] : [...tags];
}
