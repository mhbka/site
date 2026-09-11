import assert from 'node:assert/strict';
import test from 'node:test';
import { restoreScrollPosition } from '../../../../src/components/pix/scroll.ts';

test('restores both viewport coordinates after opening an image viewer', () => {
	const calls: Array<[number, number]> = [];

	restoreScrollPosition({ left: 12, top: 960 }, (left, top) => calls.push([left, top]));

	assert.deepEqual(calls, [[12, 960]]);
});
