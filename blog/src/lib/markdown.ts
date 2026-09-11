import { createMarkdownProcessor } from '@astrojs/markdown-remark';

let processor: ReturnType<typeof createMarkdownProcessor> | undefined;

/** Renders Markdown to HTML with the shared Astro processor. */
export async function renderMarkdown(markdown: string): Promise<string> {
	processor ??= createMarkdownProcessor();
	const renderer = await processor;
	return (await renderer.render(markdown)).code;
}
