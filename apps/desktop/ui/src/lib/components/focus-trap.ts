const FOCUSABLE =
	'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), summary, [tabindex]:not([tabindex="-1"])';

export function focusableIn(root: HTMLElement): HTMLElement[] {
	return [...root.querySelectorAll<HTMLElement>(FOCUSABLE)];
}

export function focusInitial(root: HTMLElement): void {
	const preferred = root.querySelector<HTMLElement>('[data-autofocus]');
	(preferred ?? focusableIn(root)[0] ?? root).focus();
}

// Tab et Maj+Tab bouclent sur les éléments du dialogue : le focus n'en sort jamais.
export function trapTab(event: KeyboardEvent, root: HTMLElement): void {
	const items = focusableIn(root);
	const first = items[0];
	const last = items[items.length - 1];
	if (first === undefined || last === undefined) {
		event.preventDefault();
		return;
	}
	const active = document.activeElement;
	const isLeavingEnd = !event.shiftKey && active === last;
	const isLeavingStart = event.shiftKey && (active === first || active === root);
	if (isLeavingEnd || !root.contains(active)) {
		event.preventDefault();
		first.focus();
	} else if (isLeavingStart) {
		event.preventDefault();
		last.focus();
	}
}
