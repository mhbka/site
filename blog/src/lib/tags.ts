/** Normalizes a tag for consistent storage and filtering. */
export function normalizeTag(value: string): string {
	return value.toLowerCase().replace(/\s+/g, '');
}

/** Appends a new normalized tag while preserving existing unique tags. */
export function addTag(tags: readonly string[], value: string): string[] {
	const tag = normalizeTag(value);
	return tag && !tags.includes(tag) ? [...tags, tag] : [...tags];
}
