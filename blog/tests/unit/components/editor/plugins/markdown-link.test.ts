import assert from 'node:assert/strict';
import test from 'node:test';
import { Schema } from '@milkdown/kit/prose/model';
import { EditorState, TextSelection } from '@milkdown/kit/prose/state';

import { convertMarkdownLink, findLinkAtPosition, insertTextAfterLink, restoreMarkdownLink } from '../../../../../src/components/editor/plugins/markdown-link.ts';

const schema = new Schema({
	nodes: {
		doc: { content: 'paragraph+' },
		paragraph: { content: 'inline*' },
		text: { group: 'inline' },
	},
	marks: {
		link: {
			attrs: { href: {}, title: { default: null } },
			inclusive: false,
		},
	},
});

/** Applies the input-rule transformation to a paragraph of Markdown. */
function applyMarkdownLink(markdown: string) {
	const state = EditorState.create({
		schema,
		doc: schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, schema.text(markdown))]),
	});
	const match = markdown.match(/\[([^\]\n]+)]\((<[^>\n]+>|[^()\s]+)(?:\s+"([^"\n]+)")?\)$/);
	if (!match) throw new Error('Test Markdown must contain a complete link.');

	const start = markdown.length - match[0].length + 1;
	const transaction = convertMarkdownLink(state, match, start, markdown.length + 1, schema.marks.link);
	return transaction?.doc;
}

test('converts typed Markdown link syntax into a link mark', () => {
	const doc = applyMarkdownLink('Read [Milkdown](https://milkdown.dev).'.slice(0, -1));
	const link = doc?.firstChild?.lastChild?.marks.find((mark) => mark.type === schema.marks.link);

	assert.equal(doc?.textContent, 'Read Milkdown');
	assert.equal(link?.attrs.href, 'https://milkdown.dev');
	assert.equal(link?.attrs.title, null);
});

test('preserves unsafe link destinations as Markdown text', () => {
	const markdown = '[Unsafe](javascript:alert)';
	const state = EditorState.create({
		schema,
		doc: schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, schema.text(markdown))]),
	});
	const match = markdown.match(/\[([^\]\n]+)]\((<[^>\n]+>|[^()\s]+)(?:\s+"([^"\n]+)")?\)$/);

	assert.ok(match);
	assert.equal(convertMarkdownLink(state, match, 1, markdown.length + 1, schema.marks.link), null);
});

test('restores a link as Markdown after backspacing within it', () => {
	const link = schema.text('Milkdown', [schema.marks.link.create({ href: 'https://milkdown.dev', title: null })]);
	const document = schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, [schema.text('Read '), link])]);
	const state = EditorState.create({
		schema,
		doc: document,
		selection: TextSelection.create(document, 7),
	});
	const transaction = restoreMarkdownLink(state, schema.marks.link);

	assert.equal(transaction?.doc.textContent, 'Read [Milkdown](https://milkdown.dev)');
	assert.equal(transaction?.selection.from, 'Read [Milkdown](https://milkdown.dev)'.length + 1);
	assert.equal(transaction?.doc.firstChild?.lastChild?.marks.length, 0);
});

test('finds a link mark at a clicked position', () => {
	const link = schema.text('Milkdown', [schema.marks.link.create({ href: 'https://milkdown.dev', title: null })]);
	const document = schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, link)]);
	const foundLink = findLinkAtPosition(document, 2, schema.marks.link);

	assert.equal(foundLink?.attrs.href, 'https://milkdown.dev');
});

test('does not extend a link mark when typing after it', () => {
	const link = schema.text('Milkdown', [schema.marks.link.create({ href: 'https://milkdown.dev', title: null })]);
	const document = schema.nodes.doc.create(null, [schema.nodes.paragraph.create(null, link)]);
	const state = EditorState.create({
		schema,
		doc: document,
		selection: TextSelection.create(document, 9),
	});
	const transaction = insertTextAfterLink(state, '!', schema.marks.link);

	assert.equal(transaction?.doc.textContent, 'Milkdown!');
	assert.equal(transaction?.doc.firstChild?.lastChild?.marks.length, 0);
});
