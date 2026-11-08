<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';
	import type { IconName } from './icons.js';
	import { focusInitial, trapTab } from './focus-trap.js';

	type Props = {
		title: string;
		description?: string;
		icon: IconName;
		kind: 'alert' | 'info';
		onescape?: () => void;
		children: Snippet;
	};

	let { title, description, icon, kind, onescape, children }: Props = $props();

	const uid = $props.id();
	let dialog: HTMLDivElement | undefined = $state();

	// Le focus va dans le dialogue à l'ouverture et revient au déclencheur à la fermeture.
	$effect(() => {
		const trigger = document.activeElement;
		if (dialog) focusInitial(dialog);
		return () => {
			if (trigger instanceof HTMLElement) trigger.focus();
		};
	});

	function onkeydown(event: KeyboardEvent): void {
		if (!dialog) return;
		if (event.key === 'Tab') trapTab(event, dialog);
		else if (event.key === 'Escape' && onescape) {
			event.stopPropagation();
			onescape();
		}
	}
</script>

<svelte:window {onkeydown} />

<div class="scrim">
	<div
		bind:this={dialog}
		class="dialog"
		role={kind === 'alert' ? 'alertdialog' : 'dialog'}
		aria-modal="true"
		aria-labelledby="{uid}-title"
		aria-describedby={description ? `${uid}-description` : undefined}
		tabindex="-1"
	>
		<div class="head">
			<div class="dialog-icon" class:dialog-icon--info={kind === 'info'}>
				<Icon name={icon} isLarge />
			</div>
			<div class="titles">
				<h2 class="h2" id="{uid}-title">{title}</h2>
				{#if description}
					<span class="muted" id="{uid}-description">{description}</span>
				{/if}
			</div>
		</div>
		{@render children()}
	</div>
</div>

<style>
	.scrim {
		position: fixed;
		z-index: 100;
	}

	.dialog {
		max-height: 100%;
		overflow-y: auto;
	}

	.head {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.titles {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}
</style>
