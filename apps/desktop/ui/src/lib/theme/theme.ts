export const THEME_PREFERENCES = ['light', 'dark', 'system'] as const;
export type ThemePreference = (typeof THEME_PREFERENCES)[number];
export type Theme = 'light' | 'dark';

export const DARK_QUERY = '(prefers-color-scheme: dark)';

export function resolveTheme(preference: ThemePreference, isSystemDark: boolean): Theme {
	switch (preference) {
		case 'light':
			return 'light';
		case 'dark':
			return 'dark';
		case 'system':
			return isSystemDark ? 'dark' : 'light';
	}
}
