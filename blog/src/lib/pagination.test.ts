import assert from 'node:assert/strict';
import test from 'node:test';

import { parsePage } from './pagination.ts';

test('uses the first page for absent or invalid page parameters', () => {
	assert.equal(parsePage(null), 1);
	assert.equal(parsePage('0'), 1);
	assert.equal(parsePage('-2'), 1);
	assert.equal(parsePage('1.5'), 1);
	assert.equal(parsePage('abc'), 1);
});

test('accepts positive integer page parameters', () => {
	assert.equal(parsePage('3'), 3);
});
