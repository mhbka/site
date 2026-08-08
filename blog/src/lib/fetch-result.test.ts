import assert from 'node:assert/strict';
import test from 'node:test';

import { fetchResult } from './fetch-result.ts';

test('returns fetched data when a request succeeds', async () => {
	const result = await fetchResult(async () => ['post'], 'Could not load posts.');

	assert.deepEqual(result, { data: ['post'], error: undefined });
});

test('returns the supplied message when a request fails', async () => {
	const result = await fetchResult(async () => Promise.reject(new Error('offline')), 'Could not load posts.');

	assert.deepEqual(result, { data: undefined, error: 'Could not load posts.' });
});
