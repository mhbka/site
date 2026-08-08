import assert from 'node:assert/strict';
import test from 'node:test';

import { renderMarkdown } from './markdown.ts';

test('renders backend Markdown as HTML', async () => {
	const html = await renderMarkdown('# Hello\n\nA **blog post**.');

	assert.match(html, /<h1 id="hello">Hello<\/h1>/);
	assert.match(html, /<strong>blog post<\/strong>/);
});
