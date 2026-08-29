import assert from 'node:assert/strict';
import test from 'node:test';
import { Schema } from '@milkdown/kit/prose/model';
import { TextSelection } from '@milkdown/kit/prose/state';
import { findImageBeforeCursor } from './image-selection.ts';

const schema = new Schema({
	nodes: {
		doc: { content: 'paragraph+' },
		paragraph: { content: 'inline*' },
		text: { group: 'inline' },
		image: { inline: true, group: 'inline', atom: true, selectable: true },
	},
});

const image = schema.nodes.image.create();

test('selects an image immediately before the cursor', () => {
	const doc = schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, [image, schema.text('text')])]);
	const selection = TextSelection.create(doc, 2);

	assert.equal(findImageBeforeCursor(selection, doc), 1);
});

test('selects an image on the visually preceding line', () => {
	const doc = schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, image), schema.nodes.paragraph.create()]);
	const selection = TextSelection.create(doc, 4);

	assert.equal(findImageBeforeCursor(selection, doc), 1);
});

test('does not intercept backspace after ordinary text', () => {
	const doc = schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, schema.text('text'))]);
	const selection = TextSelection.create(doc, 2);

	assert.equal(findImageBeforeCursor(selection, doc), undefined);
});
