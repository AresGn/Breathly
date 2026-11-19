<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import DialogShell from './DialogShell.svelte';

	type Props = {
		explanation: string;
		title?: string;
		onclose: () => void;
		onretry?: () => void;
	};

	let { explanation, title, onclose, onretry }: Props = $props();
</script>

<DialogShell
	title={title ?? t('dialog.error.title')}
	description={explanation}
	icon="alert"
	kind="alert"
	onescape={onclose}
>
	<div class="dialog-actions">
		{#if onretry}
			<button type="button" class="btn btn--secondary" onclick={onretry}>
				{t('dialog.retry')}
			</button>
		{/if}
		<button type="button" class="btn btn--primary" data-autofocus onclick={onclose}>
			{t('dialog.close')}
		</button>
	</div>
</DialogShell>
