import { marked } from 'marked';
import purify from 'dompurify';

/**
 * Full URL to the project's GitHub repository
 */
export const REPO_URL = 'https://github.com/Gawdl3y/Resolute/';

/**
 * Renders Markdown into sanitized HTMl with auto-linked GitHub issue numbers and commit hashes
 * @param {string} markdown
 * @returns {string}
 */
export function renderMarkdown(markdown) {
	// Replace git hashes with Markdown links
	markdown = markdown.replace(
		/\b(([0-9a-f]{7})([0-9a-f]{1,33})?)\b/g,
		`[$2](${REPO_URL}commit/$1)`,
	);

	// Replace issue numbers with Markdown links
	markdown = markdown.replace(
		/(\s)(#[0-9]+)\b/g,
		`$1[$2](${REPO_URL}issues/$1)`,
	);

	// Set up the marked renderer to make links open in a new window
	const renderer = new marked.Renderer();
	const linkRenderer = renderer.link;
	renderer.link = (href, title, text) => {
		const html = linkRenderer.call(renderer, href, title, text);
		return html.replace(/^<a /, '<a target="_blank" ');
	};

	// Render and sanitize the Markdown
	const rendered = marked(markdown, { renderer });
	return purify.sanitize(rendered, { ADD_ATTR: ['target'] });
}

/**
 * Disables the context menu for a node
 * @param {Node} [node=document]
 */
export function disableContextMenu(node = document) {
	node.addEventListener(
		'contextmenu',
		(evt) => {
			evt.preventDefault();
			return false;
		},
		{ capture: true },
	);
}

/**
 * Disables text selection for a node
 * @param {Node} [node=document]
 */
export function disableTextSelection(node = document) {
	node.addEventListener(
		'selectstart',
		(evt) => {
			evt.preventDefault();
			return false;
		},
		{ capture: true },
	);
}
