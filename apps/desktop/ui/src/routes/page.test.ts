import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { setLocale } from '#lib/i18n/i18n.svelte.js';
import Page from './+page.svelte';

describe('accueil (H1)', () => {
	it('affiche le titre de l’écran d’accueil', () => {
		setLocale('fr');
		render(Page);
		expect(screen.getByRole('heading', { level: 1, name: 'Accueil' })).toBeInTheDocument();
	});
});
