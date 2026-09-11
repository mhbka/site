import assert from 'node:assert/strict';
import test from 'node:test';

import { getPostEditorLabels } from '../../../src/lib/post-editor.ts';

test('shows published post labels in the editor', () => {
	assert.deepEqual(getPostEditorLabels('published'), {
		state: '[Published]',
		submit: 'Save changes',
	});
});

test('uses draft labels for drafts and new posts', () => {
	assert.deepEqual(getPostEditorLabels('draft'), {
		state: '[Draft]',
		submit: 'Save draft',
	});
	assert.deepEqual(getPostEditorLabels(undefined), {
		state: '[Draft]',
		submit: 'Save draft',
	});
});
