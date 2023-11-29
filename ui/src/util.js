/**
 * Disables the context menu for a node
 * @param {Node} [node=document]
 */
export function disableContextMenu(node = document) {
	node.addEventListener('contextmenu', evt => {
		evt.preventDefault();
		return false;
	}, { capture: true });
}

/**
 * Disables text selection for a node
 * @param {Node} [node=document]
 */
export function disableTextSelection(node = document) {
	node.addEventListener('selectstart', evt => {
		evt.preventDefault();
		return false;
	}, { capture: true });
}
