<script lang="ts">
	import { t } from '#lib/i18n/i18n.svelte.js';
	import { RISK_LEVELS, type RiskLevel } from './risk-level.js';

	let { level }: { level: RiskLevel } = $props();
	const risk = $derived(RISK_LEVELS[level]);
</script>

<span class="risk {risk.className}">
	{#if risk.icon === 'check'}
		<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
			<circle cx="12" cy="12" r="9" />
			<path d="m8 12.5 2.8 2.8L16 9.5" />
		</svg>
	{:else if risk.icon === 'eye'}
		<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
			<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z" />
			<circle cx="12" cy="12" r="2.8" />
		</svg>
	{:else}
		<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
			<circle cx="12" cy="8" r="3.5" />
			<path d="M5 20c0-3.9 3.1-7 7-7s7 3.1 7 7" />
		</svg>
	{/if}
	{t(risk.label)}
</span>
