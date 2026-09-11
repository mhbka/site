/** Parses a positive page number and defaults invalid values to one. */
export function parsePage(value: string | null): number {
	const page = Number(value);
	return Number.isSafeInteger(page) && page > 0 ? page : 1;
}
