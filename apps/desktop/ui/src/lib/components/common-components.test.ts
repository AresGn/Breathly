import { cleanup, fireEvent, render, within } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Locale } from '#lib/i18n/catalog.js';
import { resetLocale, setLocale } from '#lib/i18n/i18n.svelte.js';
import ByteSize from './ByteSize.svelte';
import EmptyState from './EmptyState.svelte';
import ErrorState from './ErrorState.svelte';
import LoadingState from './LoadingState.svelte';
import RiskLevelBadge from './RiskLevelBadge.svelte';
import UnavailableState from './UnavailableState.svelte';
import type { RiskLevel } from './risk-level.js';

const LOCALES: readonly Locale[] = ['fr', 'en'];
const EXPECTED_BADGES: readonly {
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
	it.each(EXPECTED_BADGES)(
		'affiche libellé, icône et style pour $level',
		({ level, className, fr, en }) => {
			for (const locale of LOCALES) {
				setLocale(locale);
				const { container } = render(RiskLevelBadge, { props: { level } });
				const badge = within(container)
					.getByText(locale === 'fr' ? fr : en)
					.closest('.risk');
				const icon = badge?.querySelector('svg');

				expect(badge).toHaveClass('risk', className);
				expect(icon).toHaveAttribute('aria-hidden', 'true');
				expect(icon?.querySelector('path')).toBeInTheDocument();
			}
		}
	);

	it('distingue chaque niveau par une icône différente, jamais par la couleur seule', () => {
		const icons = EXPECTED_BADGES.map(({ level }) => {
			const { container } = render(RiskLevelBadge, { props: { level } });
			return container.querySelector('svg')?.innerHTML;
		});

		expect(new Set(icons).size).toBe(EXPECTED_BADGES.length);
	});
});

describe('affichage des tailles', () => {
	it.each(LOCALES)('formate selon Intl en %s', (locale) => {
		setLocale(locale);
		const { container } = render(ByteSize, { props: { bytes: 2_500_000_000 } });

		expect(within(container).getByText(locale === 'fr' ? '2,5 Go' : '2.5 GB')).toBeInTheDocument();
	});
});

describe('états communs', () => {
	it.each(LOCALES)('chargement accessible en %s, avec squelettes', (locale) => {
		setLocale(locale);
		const { container } = render(LoadingState);
		const status = within(container).getByRole('status', {
			busy: true,
			name: locale === 'fr' ? 'Chargement…' : 'Loading…'
		});

		expect(status.querySelectorAll('.skel').length).toBeGreaterThan(0);
	});

	it.each(LOCALES)('vide expliqué en %s', (locale) => {
		setLocale(locale);
		const { container } = render(EmptyState);

		expect(container.querySelector('.empty')).toBeInTheDocument();
		expect(within(container).getByRole('status')).toHaveTextContent(
			locale === 'fr' ? 'Aucun élément' : 'Nothing to show'
		);
		expect(container.textContent).toContain(
			locale === 'fr' ? 'rien à afficher' : 'nothing to show'
		);
	});

	it.each(LOCALES)('erreur avec explication et bouton « réessayer » en %s', async (locale) => {
		setLocale(locale);
		const explanation = locale === 'fr' ? 'La mesure n’a pas pu être lue.' : 'The reading failed.';
		const onretry = vi.fn();
		const { container } = render(ErrorState, { props: { explanation, onretry } });
		const alert = within(container).getByRole('alert');

		expect(alert).toHaveClass('banner', 'banner--warn');
		expect(alert).toHaveTextContent(explanation);
		await fireEvent.click(
			within(alert).getByRole('button', { name: locale === 'fr' ? 'Réessayer' : 'Try again' })
		);
		expect(onretry).toHaveBeenCalledOnce();
	});

	it('erreur sans bouton quand aucune reprise n’est possible', () => {
		const { container } = render(ErrorState, { props: { explanation: 'Échec.' } });

		expect(within(container).queryByRole('button')).not.toBeInTheDocument();
	});

	it.each(LOCALES)('mesure non disponible sans valeur inventée en %s', (locale) => {
		setLocale(locale);
		const label = locale === 'fr' ? 'Chauffe' : 'Temperature';
		const { container } = render(UnavailableState, { props: { label } });

		expect(within(container).getByRole('status')).toHaveTextContent(label);
		expect(container.textContent).toContain(locale === 'fr' ? 'Non disponible' : 'Not available');
		expect(container.textContent).not.toMatch(/\d/);
	});
});
