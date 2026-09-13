import assert from 'node:assert/strict';
import test from 'node:test';
import { savePixTagEdits } from '../../../../src/components/pix/tag-edit.ts';

test('sends selected images, operation, and tags to the bulk updater', async () => {
	const calls: unknown[][] = [];
	const result = await savePixTagEdits({
		imageIds: ['one', 'two'],
		operation: 'add',
		tags: ['cats'],
		update: async (...args) => {
			calls.push(args);
			return [{ id: 'one', tags: ['cats'] }];
		},
	});

	assert.deepEqual(calls, [[['one', 'two'], 'add', ['cats']]]);
	assert.deepEqual(result, [{ id: 'one', tags: ['cats'] }]);
});

test('rejects an edit without a selected image or tag', async () => {
	await assert.rejects(
		savePixTagEdits({ imageIds: [], operation: 'remove', tags: ['cats'], update: async () => [] }),
		/select at least one image/,
	);
	await assert.rejects(
		savePixTagEdits({ imageIds: ['one'], operation: 'remove', tags: [], update: async () => [] }),
		/enter at least one tag/,
	);
});
