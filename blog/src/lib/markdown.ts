import { createMarkdownProcessor } from '@astrojs/markdown-remark';

let processor: ReturnType<typeof createMarkdownProcessor> | undefined;

/** Combines directly adjacent unordered lists into one continuous list. */
function mergeAdjacentUnorderedLists(html: string): string {
	return html.replace(/<\/ul>\s*<ul>/g, '');
}

/** Renders Markdown to HTML with the shared Astro processor. */
export async function renderMarkdown(markdown: string): Promise<string> {
	processor ??= createMarkdownProcessor();
	const renderer = await processor;
	return mergeAdjacentUnorderedLists((await renderer.render(markdown)).code);
}
