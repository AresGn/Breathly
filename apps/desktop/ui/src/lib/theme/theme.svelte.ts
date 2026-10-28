import { DARK_QUERY, resolveTheme, type Theme, type ThemePreference } from './theme.js';

let preference = $state<ThemePreference>('system');
let isSystemDark = $state(false);

export function getThemePreference(): ThemePreference {
	return preference;
}

export function getTheme(): Theme {
	return resolveTheme(preference, isSystemDark);
}

export function setThemePreference(next: ThemePreference): void {
	preference = next;
	applyTheme();
}

function applyTheme(): void {
	document.documentElement.dataset['theme'] = getTheme();
}

// Applique le thème et suit l'OS quand il change ; retourne la fonction qui arrête le suivi.
export function startThemeSync(): () => void {
	const query = window.matchMedia(DARK_QUERY);
	const onChange = (event: MediaQueryListEvent): void => {
		isSystemDark = event.matches;
		applyTheme();
	};
	isSystemDark = query.matches;
	applyTheme();
	query.addEventListener('change', onChange);
	return () => query.removeEventListener('change', onChange);
}
