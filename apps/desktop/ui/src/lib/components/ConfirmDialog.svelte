<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import DialogShell from './DialogShell.svelte';

	type Props = {
		title: string;
		description?: string;
		confirmLabel?: string;
		onconfirm: () => void;
		oncancel: () => void;
	};

	let { title, description, confirmLabel, onconfirm, oncancel }: Props = $props();
</script>

<DialogShell {title} {description} icon="info" kind="info" onescape={oncancel}>
	<div class="dialog-actions">
		<button type="button" class="btn btn--secondary" data-autofocus onclick={oncancel}>
			{t('dialog.cancel')}
		</button>
		<button type="button" class="btn btn--primary" onclick={onconfirm}>
			{confirmLabel ?? t('dialog.confirm')}
		</button>
	</div>
</DialogShell>
