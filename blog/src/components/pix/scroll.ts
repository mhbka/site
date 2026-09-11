/** Represents the document viewport position before opening the image viewer. */
export interface ScrollPosition {
	left: number;
	top: number;
}

/** Restores a previously saved document viewport position. */
export function restoreScrollPosition(position: ScrollPosition, scrollTo: (left: number, top: number) => void) {
	scrollTo(position.left, position.top);
}
