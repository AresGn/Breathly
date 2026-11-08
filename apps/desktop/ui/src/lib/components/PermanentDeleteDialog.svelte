<script lang="ts">
	import type { Snippet } from 'svelte';
	import { t } from '#lib/i18n/i18n.svelte.js';
	import DialogShell from './DialogShell.svelte';

	type Props = {
		title: string;
		summary?: string;
		confirmLabel: string;
		list?: Snippet;
		onconfirm: () => void;
		oncancel: () => void;
	};

	let { title, summary, confirmLabel, list, onconfirm, oncancel }: Props = $props();
</script>

<DialogShell {title} description={summary} icon="trash" kind="alert" onescape={oncancel}>
	<p class="warning">{t('dialog.permanent.warning')}</p>
	{#if list}
		<details>
			<summary>{t('dialog.permanent.showList')}</summary>
			{@render list()}
		</details>
	{/if}
	<div class="dialog-actions">
		<button type="button" class="btn btn--secondary" data-autofocus onclick={oncancel}>
			{t('dialog.cancel')}
		</button>
		<button type="button" class="btn btn--danger" onclick={onconfirm}>
			{confirmLabel}
		</button>
	</div>
</DialogShell>

<style>
	.warning {
		margin: 0;
		font-weight: 600;
	}
</style>
