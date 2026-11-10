import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Locale } from '#lib/i18n/catalog.js';
import { resetLocale, setLocale } from '#lib/i18n/i18n.svelte.js';
import CommandCopyDialog from './CommandCopyDialog.svelte';
import ConfirmDialog from './ConfirmDialog.svelte';
import ErrorDialog from './ErrorDialog.svelte';
import PermanentDeleteDialog from './PermanentDeleteDialog.svelte';

const LOCALES: readonly Locale[] = ['fr', 'en'];
const COMMAND = 'sudo apt install zram-tools';

afterEach(() => {
	cleanup();
	resetLocale();
	vi.unstubAllGlobals();
});

function stubClipboard(writeText: (text: string) => Promise<void>): void {
	vi.stubGlobal('navigator', { ...navigator, languages: ['fr'], clipboard: { writeText } });
}

describe('dialogue de confirmation', () => {
	it.each(LOCALES)('a un nom, une description et des boutons nommés en %s', (locale) => {
		setLocale(locale);
		render(ConfirmDialog, {
			props: { title: 'Titre', description: 'Détail', onconfirm: vi.fn(), oncancel: vi.fn() }
		});
		const dialog = screen.getByRole('dialog', { name: 'Titre' });

		expect(dialog).toHaveAttribute('aria-modal', 'true');
		expect(dialog).toHaveAccessibleDescription('Détail');
		expect(
			within(dialog).getByRole('button', { name: locale === 'fr' ? 'Annuler' : 'Cancel' })
		).toBeInTheDocument();
		expect(
			within(dialog).getByRole('button', { name: locale === 'fr' ? 'Confirmer' : 'Confirm' })
		).toBeInTheDocument();
	});

	it('émet la confirmation ou l’annulation', async () => {
		setLocale('fr');
		const onconfirm = vi.fn();
		const oncancel = vi.fn();
		render(ConfirmDialog, { props: { title: 'T', onconfirm, oncancel } });

		await fireEvent.click(screen.getByRole('button', { name: 'Confirmer' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Annuler' }));

		expect(onconfirm).toHaveBeenCalledOnce();
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('Échap annule', async () => {
		const oncancel = vi.fn();
		render(ConfirmDialog, { props: { title: 'T', onconfirm: vi.fn(), oncancel } });

		await fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });

		expect(oncancel).toHaveBeenCalledOnce();
	});
});

describe('focus', () => {
	it('piège le focus, le place sur le bouton sûr et le rend au déclencheur', async () => {
		const trigger = document.createElement('button');
		document.body.append(trigger);
		trigger.focus();
		setLocale('fr');
		const { unmount } = render(ConfirmDialog, {
			props: { title: 'T', onconfirm: vi.fn(), oncancel: vi.fn() }
		});
		const dialog = screen.getByRole('dialog');
		const cancel = screen.getByRole('button', { name: 'Annuler' });
		const confirm = screen.getByRole('button', { name: 'Confirmer' });

		await waitFor(() => expect(cancel).toHaveFocus());
		confirm.focus();
		await fireEvent.keyDown(dialog, { key: 'Tab' });
		expect(cancel).toHaveFocus();
		await fireEvent.keyDown(dialog, { key: 'Tab', shiftKey: true });
		expect(confirm).toHaveFocus();

		unmount();
		await waitFor(() => expect(trigger).toHaveFocus());
		trigger.remove();
	});

	it('retient Tab et Échap même lorsque le focus est sorti du dialogue', async () => {
		const outside = document.createElement('button');
		document.body.append(outside);
		const oncancel = vi.fn();
		setLocale('fr');
		render(ConfirmDialog, { props: { title: 'T', onconfirm: vi.fn(), oncancel } });

		try {
			outside.focus();
			await fireEvent.keyDown(outside, { key: 'Tab' });
			expect(screen.getByRole('button', { name: 'Annuler' })).toHaveFocus();

			outside.focus();
			await fireEvent.keyDown(outside, { key: 'Escape' });
			expect(oncancel).toHaveBeenCalledOnce();
		} finally {
			outside.remove();
		}
	});
});

describe('avertissement de suppression définitive', () => {
	it.each(LOCALES)('dit que c’est définitif en %s, bouton destructif', (locale) => {
		setLocale(locale);
		render(PermanentDeleteDialog, {
			props: {
				title: 'Vider',
				summary: '214',
				confirmLabel: 'Vider la corbeille',
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		const dialog = screen.getByRole('alertdialog', { name: 'Vider' });

		expect(dialog).toHaveTextContent(locale === 'fr' ? 'Suppression définitive' : 'Permanent');
		expect(within(dialog).getByRole('button', { name: 'Vider la corbeille' })).toHaveClass(
			'btn--danger'
		);
	});

	it('place le focus sur Annuler, jamais sur la suppression', async () => {
		setLocale('fr');
		render(PermanentDeleteDialog, {
			props: { title: 'V', confirmLabel: 'Vider', onconfirm: vi.fn(), oncancel: vi.fn() }
		});

		await waitFor(() => expect(screen.getByRole('button', { name: 'Annuler' })).toHaveFocus());
	});

	it.each(LOCALES)('permet d’ouvrir la liste avant les actions en %s', async (locale) => {
		setLocale(locale);
		const list = createRawSnippet(() => ({
			render: () => '<ul><li>Élément consultable</li></ul>'
		}));
		render(PermanentDeleteDialog, {
			props: {
				title: 'V',
				confirmLabel: 'Vider',
				list,
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		const dialog = screen.getByRole('alertdialog');
		const summary = within(dialog).getByText(locale === 'fr' ? 'Voir la liste' : 'View the list');
		const details = summary.closest('details');

		expect(details).not.toBeNull();
		expect(details).not.toHaveAttribute('open');
		await fireEvent.click(summary);
		expect(within(dialog).getByText('Élément consultable')).toBeVisible();
	});
});

describe('message d’erreur expliqué', () => {
	it.each(LOCALES)('annonce l’explication et se ferme en %s', async (locale) => {
		setLocale(locale);
		const onclose = vi.fn();
		render(ErrorDialog, { props: { explanation: 'Disque plein.', onclose } });
		const dialog = screen.getByRole('alertdialog', {
			name: locale === 'fr' ? 'Une erreur est survenue' : 'Something went wrong'
		});

		expect(dialog).toHaveTextContent('Disque plein.');
		expect(dialog).toHaveAccessibleDescription('Disque plein.');
		expect(within(dialog).queryByRole('button', { name: /try again|réessayer/i })).toBeNull();
		await fireEvent.click(
			within(dialog).getByRole('button', { name: locale === 'fr' ? 'Fermer' : 'Close' })
		);
		expect(onclose).toHaveBeenCalledOnce();
	});

	it('propose de réessayer quand c’est possible', async () => {
		setLocale('fr');
		const onretry = vi.fn();
		render(ErrorDialog, { props: { explanation: 'x', onclose: vi.fn(), onretry } });

		await fireEvent.click(screen.getByRole('button', { name: 'Réessayer' }));

		expect(onretry).toHaveBeenCalledOnce();
	});
});

describe('aide à la copie de commande', () => {
	function renderCommandDialog(rollbackCommand?: string): HTMLDetailsElement {
		render(CommandCopyDialog, {
			props: {
				title: 'Régler',
				explanation: 'Pourquoi.',
				command: COMMAND,
				rollbackCommand,
				onclose: vi.fn()
			}
		});
		const details = document.querySelector('details');
		if (!details) throw new Error('bloc « Voir la commande » absent');
		return details;
	}

	it('replie la commande par défaut et la montre exacte une fois dépliée', () => {
		setLocale('fr');
		const details = renderCommandDialog();

		expect(details.open).toBe(false);
		expect(within(details).getByText('Voir la commande')).toBeInTheDocument();
		expect(details.querySelector('.code')?.textContent).toBe(COMMAND);
	});

	it.each(LOCALES)('copie exactement la commande en %s', async (locale) => {
		setLocale(locale);
		const writeText = vi.fn().mockResolvedValue(undefined);
		stubClipboard(writeText);
		renderCommandDialog();

		await fireEvent.click(
			screen.getByRole('button', {
				name: locale === 'fr' ? 'Copier la commande' : 'Copy the command'
			})
		);

		expect(writeText).toHaveBeenCalledExactlyOnceWith(COMMAND);
		expect(await screen.findByRole('status')).toHaveTextContent(
			locale === 'fr' ? 'Commande copiée' : 'Command copied'
		);
	});

	it('signale l’échec de la copie et déplie la commande', async () => {
		stubClipboard(vi.fn().mockRejectedValue(new Error('refusé')));
		setLocale('fr');
		const details = renderCommandDialog();

		await fireEvent.click(screen.getByRole('button', { name: 'Copier la commande' }));

		expect(await screen.findByRole('status')).toHaveTextContent('La copie a échoué');
		await waitFor(() => expect(details.open).toBe(true));
	});

	it('garde la commande dépliée après une copie réussie', async () => {
		const writeText = vi
			.fn()
			.mockRejectedValueOnce(new Error('refusé'))
			.mockResolvedValue(undefined);
		stubClipboard(writeText);
		const details = renderCommandDialog();

		await fireEvent.click(screen.getByRole('button', { name: 'Copier la commande' }));
		await waitFor(() => expect(details.open).toBe(true));
		await fireEvent.click(screen.getByRole('button', { name: 'Copier la commande' }));

		expect(await screen.findByRole('status')).toHaveTextContent('Commande copiée');
		expect(details.open).toBe(true);
	});

	it.each(LOCALES)('affiche la commande optionnelle pour revenir en arrière en %s', (locale) => {
		setLocale(locale);
		const rollbackCommand = 'sudo apt remove zram-tools';
		const details = renderCommandDialog(rollbackCommand);

		expect(
			within(details).getByText(
				locale === 'fr' ? 'Pour revenir en arrière plus tard :' : 'To undo this later:'
			)
		).toBeInTheDocument();
		expect(within(details).getByText(rollbackCommand)).toBeInTheDocument();
	});
});
