import { render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { resetLocale, setLocale } from '#lib/i18n/i18n.svelte.js';
import ScreenPage from './ScreenPage.svelte';
import Sidebar from './Sidebar.svelte';

afterEach(resetLocale);

describe('barre latérale', () => {
	it('expose une navigation nommée avec les cinq entrées dans l’ordre', () => {
		setLocale('fr');
		render(Sidebar, { pathname: '/' });
		const nav = screen.getByRole('navigation', { name: 'Navigation principale' });
		const links = Array.from(nav.querySelectorAll('a'));
		expect(links.map((link) => link.textContent)).toEqual([
			'Accueil',
			'Diagnostic',
			'Espace disque',
			'Historique',
			'Réglages'
		]);
	});

	it('marque la page courante', () => {
		setLocale('fr');
		render(Sidebar, { pathname: '/diagnostic' });
		expect(screen.getByRole('link', { name: 'Diagnostic' })).toHaveAttribute(
			'aria-current',
			'page'
		);
		expect(screen.getByRole('link', { name: 'Accueil' })).not.toHaveAttribute('aria-current');
	});

	it('garde la section active sur un sous-écran', () => {
		setLocale('fr');
		render(Sidebar, { pathname: '/nettoyage/resume' });
		expect(screen.getByRole('link', { name: 'Espace disque' })).toHaveAttribute(
			'aria-current',
			'true'
		);
	});

	it('suit la langue', () => {
		setLocale('en');
		render(Sidebar, { pathname: '/' });
		expect(screen.getByRole('link', { name: 'Settings' })).toBeInTheDocument();
	});

	it('permet d’atteindre chaque entrée au clavier', () => {
		render(Sidebar, { pathname: '/' });
		for (const link of screen.getAllByRole('link')) expect(link.tabIndex).toBeGreaterThanOrEqual(0);
	});
});

describe('page d’écran', () => {
	it('affiche le titre traduit de l’écran', () => {
		setLocale('fr');
		render(ScreenPage, { id: 'N4' });
		expect(screen.getByRole('heading', { level: 1, name: 'Résumé du nettoyage' })).toBeVisible();
	});

	it('place le focus sur le titre à l’affichage', () => {
		render(ScreenPage, { id: 'D1' });
		expect(screen.getByRole('heading', { level: 1 })).toHaveFocus();
	});
});
