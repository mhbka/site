import assert from 'node:assert/strict';
import test from 'node:test';
import { uploadPixFiles } from './upload.ts';

test('uploads every selected pix with the shared tags and reports progress', async () => {
	const files = [new File(['one'], 'one.png', { type: 'image/png' }), new File(['two'], 'two.png', { type: 'image/png' })];
	const calls: Array<{ file: string; tags: string[] }> = [];
	const progress: string[] = [];

	const result = await uploadPixFiles({
		files,
		tags: ['cats'],
		upload: async (file, tags) => {
			calls.push({ file: file.name, tags });
			return { id: file.name, publicUrl: file.name, tags, createdAt: '2026-01-01T00:00:00Z' };
		},
		onProgress: ({ uploaded: count, total }) => progress.push(`${count}/${total}`),
	});

	assert.deepEqual(calls, [{ file: 'one.png', tags: ['cats'] }, { file: 'two.png', tags: ['cats'] }]);
	assert.deepEqual(progress, ['0/2', '1/2', '2/2']);
	assert.deepEqual(result, { uploaded: 2, total: 2, failed: 0 });
});

test('continues uploading after a failed pix', async () => {
	const files = [new File(['bad'], 'bad.png'), new File(['good'], 'good.png')];
	const result = await uploadPixFiles({
		files,
		tags: ['cats'],
		upload: async (file) => {
			if (file.name === 'bad.png') throw new Error('upload failed');
			return { id: file.name, publicUrl: file.name, tags: [], createdAt: '' };
		},
		onProgress: () => {},
	});

	assert.deepEqual(result, { uploaded: 1, total: 2, failed: 1 });
});
