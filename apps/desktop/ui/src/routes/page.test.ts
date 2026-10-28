import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import Page from './+page.svelte';

describe('coquille de l’interface', () => {
	it('affiche le nom du produit comme titre de la page', () => {
		render(Page);
		expect(screen.getByRole('heading', { level: 1, name: 'Breathly' })).toBeInTheDocument();
	});

	it('affiche la phrase d’accroche traduite', () => {
		render(Page);
		expect(screen.getByText(/PC/)).toBeInTheDocument();
	});
});
