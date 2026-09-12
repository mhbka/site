import assert from 'node:assert/strict';
import test from 'node:test';

import { renderMarkdown } from '../../../src/lib/markdown.ts';

test('renders backend Markdown as HTML', async () => {
	const html = await renderMarkdown('# Hello\n\nA **blog post**.');

	assert.match(html, /<h1 id="hello">Hello<\/h1>/);
	assert.match(html, /<strong>blog post<\/strong>/);
});

test('combines directly adjacent unordered lists', async () => {
	const html = await renderMarkdown('<ul>\n<li>First</li>\n</ul>\n\n<ul>\n<li>Second</li>\n</ul>');

	assert.equal((html.match(/<ul>/g) ?? []).length, 1);
	assert.match(html, /<li>First<\/li>\s*<li>Second<\/li>/);
});

test('keeps unordered lists separated by content distinct', async () => {
	const html = await renderMarkdown('- First\n\nBetween lists.\n\n- Second');

	assert.equal((html.match(/<ul>/g) ?? []).length, 2);
});
