import { cleanup, render, within } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import type { Locale } from '#lib/i18n/catalog.js';
import { resetLocale, setLocale } from '#lib/i18n/i18n.svelte.js';
import type { Theme } from '#lib/theme/theme.js';
import ByteSize from './ByteSize.svelte';
import EmptyState from './EmptyState.svelte';
import ErrorState from './ErrorState.svelte';
import LoadingState from './LoadingState.svelte';
import RiskLevelBadge from './RiskLevelBadge.svelte';
import UnavailableState from './UnavailableState.svelte';
import type { RiskLevel } from './risk-level.js';

const LOCALES_AND_THEMES: readonly { locale: Locale; theme: Theme }[] = [
	{ locale: 'fr', theme: 'light' },
	{ locale: 'fr', theme: 'dark' },
	{ locale: 'en', theme: 'light' },
	{ locale: 'en', theme: 'dark' }
];
const RISK_LEVELS: readonly {
	level: RiskLevel;
	className: string;
	fr: string;
	en: string;
}[] = [
	{ level: 'safe', className: 'risk--safe', fr: 'Sans risque', en: 'Safe' },
	{ level: 'review', className: 'risk--verify', fr: 'À vérifier', en: 'Review' },
	{
		level: 'personal',
		className: 'risk--personal',
		fr: 'Décision personnelle',
		en: 'Personal decision'
	}
];

afterEach(() => {
	cleanup();
	resetLocale();
});

describe('badge de niveau de risque', () => {
	it.each(RISK_LEVELS)(
		'affiche libellé, icône et style pour $level',
		({ level, className, fr, en }) => {
			for (const { locale, theme } of LOCALES_AND_THEMES) {
				setLocale(locale);
				const { container } = render(RiskLevelBadge, { props: { level } });
				container.dataset['theme'] = theme;
				const badge = within(container)
					.getByText(locale === 'fr' ? fr : en)
					.closest('.risk');
				const icon = badge?.querySelector('svg');

				expect(container).toHaveAttribute('data-theme', theme);
				expect(badge).toHaveClass('risk', className);
				expect(icon).toHaveAttribute('aria-hidden', 'true');
				expect(icon?.querySelector('path, circle')).toBeInTheDocument();
			}
		}
	);
});

describe('affichage des tailles', () => {
	it.each(LOCALES_AND_THEMES)(
		'formate selon Intl en $locale / thème $theme',
		({ locale, theme }) => {
			setLocale(locale);
			const { container } = render(ByteSize, { props: { bytes: 2_500_000_000 } });
			container.dataset['theme'] = theme;
			const output = within(container).getByText(locale === 'fr' ? '2,5 Go' : '2.5 GB');

			expect(container).toHaveAttribute('data-theme', theme);
			expect(output).toBeInTheDocument();
		}
	);
});

describe('états communs', () => {
	it.each(LOCALES_AND_THEMES)(
		'chargement accessible en $locale / thème $theme',
		({ locale, theme }) => {
			setLocale(locale);
			const { container } = render(LoadingState);
			container.dataset['theme'] = theme;
			const status = within(container).getByRole('status', { busy: true });

			expect(container).toHaveAttribute('data-theme', theme);
			expect(status).toHaveTextContent(locale === 'fr' ? 'Chargement…' : 'Loading…');
		}
	);

	it.each(LOCALES_AND_THEMES)('vide expliqué en $locale / thème $theme', ({ locale, theme }) => {
		setLocale(locale);
		const { container } = render(EmptyState);
		container.dataset['theme'] = theme;

		expect(container).toHaveAttribute('data-theme', theme);
		expect(within(container).getByRole('status')).toHaveTextContent(
			locale === 'fr' ? 'Aucun élément' : 'Nothing to show'
		);
		expect(container.textContent).toContain(
			locale === 'fr' ? 'rien à afficher' : 'nothing to show'
		);
	});

	it.each(LOCALES_AND_THEMES)(
		'erreur avec explication en $locale / thème $theme',
		({ locale, theme }) => {
			setLocale(locale);
			const explanation =
				locale === 'fr' ? 'La mesure n’a pas pu être lue.' : 'The reading failed.';
			const { container } = render(ErrorState, { props: { explanation } });
			container.dataset['theme'] = theme;

			expect(container).toHaveAttribute('data-theme', theme);
			expect(within(container).getByRole('alert')).toHaveTextContent(explanation);
		}
	);

	it.each(LOCALES_AND_THEMES)(
		'mesure non disponible sans valeur inventée en $locale / thème $theme',
		({ locale, theme }) => {
			setLocale(locale);
			const label = locale === 'fr' ? 'Chauffe' : 'Temperature';
			const { container } = render(UnavailableState, { props: { label } });
			container.dataset['theme'] = theme;

			expect(container).toHaveAttribute('data-theme', theme);
			expect(within(container).getByRole('status')).toHaveTextContent(label);
			expect(container.textContent).toContain(locale === 'fr' ? 'Non disponible' : 'Not available');
			expect(container.textContent).not.toMatch(/\d/);
		}
	);
});
