import { linkSchema, sanitizeLinkHref } from '@milkdown/kit/preset/commonmark';
import { InputRule } from '@milkdown/kit/prose/inputrules';
import type { Mark, MarkType, Node as ProseMirrorNode } from '@milkdown/kit/prose/model';
import { Plugin, PluginKey, TextSelection, type EditorState, type Selection, type Transaction } from '@milkdown/kit/prose/state';
import { $inputRule, $prose } from '@milkdown/kit/utils';

const markdownLinkPattern = /\[([^\]\n]+)]\((<[^>\n]+>|[^()\s]+)(?:\s+"([^"\n]+)")?\)$/;

interface LinkRange {
	from: number;
	to: number;
	mark: Mark;
}

/** Converts a completed Markdown link into an editor link mark. */
export function convertMarkdownLink(
	state: EditorState,
	match: RegExpMatchArray,
	start: number,
	end: number,
	linkType: MarkType,
): Transaction | null {
	const [, label, rawHref, title] = match;
	const unwrappedHref = rawHref.startsWith('<') ? rawHref.slice(1, -1) : rawHref;
	const href = sanitizeLinkHref(unwrappedHref);

	// Preserve Markdown that does not contain a safe link destination.
	if (!href) return null;

	const transaction = state.tr.insertText(label, start, end);
	transaction.addMark(start, start + label.length, linkType.create({ href, title: title ?? null }));
	transaction.setStoredMarks([]);
	return transaction;
}

/** Converts completed Markdown links while the author is typing. */
export const markdownLinkInputRule = $inputRule((ctx) => new InputRule(
	markdownLinkPattern,
	(state, match, start, end) => convertMarkdownLink(state, match, start, end, linkSchema.type(ctx)),
));

/** Finds the link mark surrounding an empty text cursor. */
function findLinkAtCursor(selection: Selection, linkType: MarkType): LinkRange | undefined {
	if (!selection.empty || selection.$from.depth === 0) return undefined;

	const parent = selection.$from.parent;
	const parentStart = selection.$from.start();
	let currentRange: LinkRange | undefined;
	let cursorRange: LinkRange | undefined;

	parent.forEach((node, offset) => {
		const mark = node.marks.find((candidate) => candidate.type === linkType);
		const from = parentStart + offset;
		const to = from + node.nodeSize;

		if (!mark) {
			currentRange = undefined;
			return;
		}
		if (!currentRange || !currentRange.mark.eq(mark) || currentRange.to !== from) {
			currentRange = { from, to, mark };
		} else {
			currentRange.to = to;
		}
		if (selection.from >= currentRange.from && selection.from <= currentRange.to) cursorRange = currentRange;
	});

	return cursorRange;
}

/** Finds the link mark at an editor document position. */
export function findLinkAtPosition(document: ProseMirrorNode, position: number, linkType: MarkType): Mark | undefined {
	const resolvedPosition = document.resolve(position);
	const marks = [
		...resolvedPosition.marks(),
		...(resolvedPosition.nodeBefore?.marks ?? []),
		...(resolvedPosition.nodeAfter?.marks ?? []),
	];
	return marks.find((mark) => mark.type === linkType);
}

/** Restores an editor link to Markdown when its text cursor is backspaced. */
export function restoreMarkdownLink(state: EditorState, linkType: MarkType): Transaction | null {
	const linkRange = findLinkAtCursor(state.selection, linkType);
	if (!linkRange) return null;

	const label = state.doc.textBetween(linkRange.from, linkRange.to);
	const title = linkRange.mark.attrs.title ? ` \"${linkRange.mark.attrs.title}\"` : '';
	const markdown = `[${label}](${linkRange.mark.attrs.href}${title})`;
	const transaction = state.tr.insertText(markdown, linkRange.from, linkRange.to);
	// Prevent the restored Markdown from inheriting the original link mark.
	transaction.removeMark(linkRange.from, linkRange.from + markdown.length, linkType);
	transaction.setSelection(TextSelection.create(transaction.doc, linkRange.from + markdown.length));
	return transaction;
}

/** Inserts ordinary text after a link without extending its link mark. */
export function insertTextAfterLink(state: EditorState, text: string, linkType: MarkType): Transaction | null {
	const linkRange = findLinkAtCursor(state.selection, linkType);
	if (!linkRange || state.selection.from !== linkRange.to) return null;

	const transaction = state.tr.insertText(text);
	transaction.removeMark(state.selection.from, state.selection.from + text.length, linkType);
	return transaction;
}

/** Restores Markdown links before allowing the author to edit their syntax. */
export const markdownLinkBackspacePlugin = $prose((ctx) => {
	const linkType = linkSchema.type(ctx);

	return new Plugin({
		key: new PluginKey('postEditorMarkdownLinkBackspace'),
		props: {
			handleTextInput(view, from, to, text) {
				if (from !== to) return false;

				const transaction = insertTextAfterLink(view.state, text, linkType);
				if (!transaction) return false;
				view.dispatch(transaction);
				return true;
			},
			handleClick(view, position, event) {
				if (!event.ctrlKey && !event.metaKey) return false;

				const link = findLinkAtPosition(view.state.doc, position, linkType);
				const href = sanitizeLinkHref(link?.attrs.href);
				if (!href) return false;

				event.preventDefault();
				window.open(href, '_blank', 'noopener,noreferrer');
				return true;
			},
			handleKeyDown(view, event) {
				if (event.key !== 'Backspace') return false;

				const transaction = restoreMarkdownLink(view.state, linkType);
				if (!transaction) return false;
				view.dispatch(transaction);
				return true;
			},
		},
	});
});
