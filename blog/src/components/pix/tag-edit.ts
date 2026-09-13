import type { PixTagUpdate } from '../../lib/api.ts';

export type PixTagOperation = 'add' | 'remove' | 'overwrite';

interface SavePixTagEditsOptions {
	imageIds: readonly string[];
	operation: PixTagOperation;
	tags: readonly string[];
	update: (imageIds: string[], operation: PixTagOperation, tags: string[]) => Promise<PixTagUpdate[]>;
}

/** Validates and sends a bulk Pix tag edit request. */
export async function savePixTagEdits({ imageIds, operation, tags, update }: SavePixTagEditsOptions): Promise<PixTagUpdate[]> {
	if (!imageIds.length) throw new Error('select at least one image.');
	if (!tags.length) throw new Error('enter at least one tag.');
	return update([...imageIds], operation, [...tags]);
}
