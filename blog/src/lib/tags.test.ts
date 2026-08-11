import assert from 'node:assert/strict';
import test from 'node:test';

import { addTag, normalizeTag } from './tags.ts';

test('normalizes tags to lowercase without spacing', () => {
	assert.equal(normalizeTag('Java Script'), 'javascript');
});

test('adds a normalized tag once and ignores empty or duplicate tags', () => {
	assert.deepEqual(addTag(['astro'], 'MilkDown'), ['astro', 'milkdown']);
	assert.deepEqual(addTag(['astro'], 'ASTRO'), ['astro']);
	assert.deepEqual(addTag(['astro'], '   '), ['astro']);
});
