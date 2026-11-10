<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import CommandCopy from './CommandCopy.svelte';
	import DialogShell from './DialogShell.svelte';

	type Props = {
		title: string;
		explanation: string;
		command: string;
		rollbackCommand?: string;
		onclose: () => void;
	};

	let { title, explanation, command, rollbackCommand, onclose }: Props = $props();
</script>

<DialogShell {title} description={explanation} icon="info" kind="info" onescape={onclose}>
	<div class="banner banner--info">
		<div class="grow">
			<span class="banner-title">{t('dialog.command.notice.title')}</span>
			{t('dialog.command.notice')}
		</div>
	</div>
	<CommandCopy {command} {rollbackCommand} />
	<div class="dialog-actions">
		<button type="button" class="btn btn--secondary" onclick={onclose}>
			{t('dialog.close')}
		</button>
	</div>
</DialogShell>
