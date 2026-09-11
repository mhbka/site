import type { Node as ProseMirrorNode } from '@milkdown/kit/prose/model';
import { NodeSelection, Plugin, PluginKey, type Selection } from '@milkdown/kit/prose/state';
import { $prose } from '@milkdown/kit/utils';

/** Reports whether a ProseMirror node is an image. */
function isImage(node: ProseMirrorNode | null | undefined) {
	return node?.type.name === 'image';
}

/** Finds an image immediately before an empty text cursor. */
export function findImageBeforeCursor(selection: Selection, doc: ProseMirrorNode): number | undefined {
	if (!selection.empty || selection.$from.depth === 0) return undefined;

	const imageBeforeCursor = selection.$from.nodeBefore;
	if (imageBeforeCursor && isImage(imageBeforeCursor)) return selection.from - imageBeforeCursor.nodeSize;
	if (selection.$from.parentOffset !== 0) return undefined;

	const currentBlockStart = selection.$from.before(selection.$from.depth);
	const previousNode = doc.resolve(currentBlockStart).nodeBefore;
	if (previousNode && isImage(previousNode)) return currentBlockStart - previousNode.nodeSize;
	if (previousNode?.type.name !== 'paragraph' || previousNode.childCount !== 1 || !isImage(previousNode.firstChild)) return undefined;
	return currentBlockStart - previousNode.nodeSize + 1;
}

/** Resolves the image position at or directly before a click. */
function findImageAtPosition(doc: ProseMirrorNode, position: number): number | undefined {
	if (isImage(doc.nodeAt(position))) return position;
	return isImage(doc.nodeAt(position - 1)) ? position - 1 : undefined;
}

/** Selects images clicked or reached with Backspace in the editor. */
export const imageSelectionPlugin = $prose(() => new Plugin({
	key: new PluginKey('postEditorImageSelection'),
	props: {
		handleClick(view, position, event) {
			if (!(event.target instanceof Element) || !event.target.closest('img')) return false;

			const imagePosition = findImageAtPosition(view.state.doc, position);
			if (imagePosition === undefined) return false;
			view.dispatch(view.state.tr.setSelection(NodeSelection.create(view.state.doc, imagePosition)));
			return true;
		},
		handleKeyDown(view, event) {
			if (event.key !== 'Backspace') return false;

			const imagePosition = findImageBeforeCursor(view.state.selection, view.state.doc);
			if (imagePosition === undefined) return false;
			view.dispatch(view.state.tr.setSelection(NodeSelection.create(view.state.doc, imagePosition)));
			return true;
		},
	},
}));
