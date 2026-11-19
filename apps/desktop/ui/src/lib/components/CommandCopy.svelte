<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import Icon from './Icon.svelte';

	type CopyState = 'idle' | 'copied' | 'failed';

	let { command, rollbackCommand }: { command: string; rollbackCommand?: string } = $props();

	let copyState = $state<CopyState>('idle');
	let isOpen = $state(false);

	async function copy(): Promise<void> {
		try {
			await navigator.clipboard.writeText(command);
			copyState = 'copied';
		} catch {
			// Presse-papiers refusé : on le dit et on déplie la commande pour la copier à la main.
			copyState = 'failed';
			isOpen = true;
		}
	}
</script>

<div class="stack">
	<div class="row row--wrap">
		<button type="button" class="btn btn--primary" data-autofocus onclick={copy}>
			<Icon name="copy" />
			{copyState === 'copied' ? t('dialog.command.copyAgain') : t('dialog.command.copy')}
		</button>
		<span role="status">
			{#if copyState === 'copied'}
				<span class="status status--ok">
					<Icon name="check" />{t('dialog.command.copied')}
				</span>
			{:else if copyState === 'failed'}
				<span class="status status--warn">
					<Icon name="alert" />{t('dialog.command.failed')}
				</span>
			{/if}
		</span>
	</div>
	<details bind:open={isOpen}>
		<summary><Icon name="chevron" />{t('dialog.command.show')}</summary>
		<div class="stack">
			<div class="code">{command}</div>
			{#if rollbackCommand}
				<div class="small muted">
					{t('dialog.command.rollback')}
					<code>{rollbackCommand}</code>
				</div>
			{/if}
		</div>
	</details>
</div>
