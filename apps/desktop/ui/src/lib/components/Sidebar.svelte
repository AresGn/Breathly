<script lang="ts">
	import { resolve } from '$app/paths';
	import { APP_NAME } from '#lib/brand.js';
	import { t } from '#lib/i18n/i18n.svelte.js';
	import { NAV_ENTRIES, normalizePath, sectionOfPath } from '#lib/navigation/screens.js';

	let { pathname }: { pathname: string } = $props();

	const currentPath = $derived(normalizePath(pathname));
	const currentSection = $derived(sectionOfPath(pathname));

	function ariaCurrent(
		entryPath: string,
		isCurrentSection: boolean
	): 'page' | 'true' | undefined {
		if (entryPath === currentPath) return 'page';
		return isCurrentSection ? 'true' : undefined;
	}
</script>

<aside>
	<p class="brand">{APP_NAME}</p>
	<p class="tagline">{t('app.tagline')}</p>
	<nav aria-label={t('nav.label')}>
		<ul>
			{#each NAV_ENTRIES as entry (entry.section)}
				{@const isCurrentSection = entry.section === currentSection}
				<li>
					<a
						href={resolve(entry.path)}
						class:active={isCurrentSection}
						aria-current={ariaCurrent(entry.path, isCurrentSection)}
						>{t(entry.labelKey)}</a
					>
				</li>
			{/each}
		</ul>
	</nav>
</aside>

<style>
	aside {
		width: var(--sidebar-w);
		flex: none;
		padding: var(--space-5) var(--space-3);
		background: var(--color-sidebar);
		border-right: 1px solid var(--color-border);
	}

	.brand {
		margin: 0;
		font-family: var(--font-display);
		font-size: var(--text-xl);
		line-height: var(--leading-tight);
		color: var(--color-accent);
	}

	.tagline {
		margin: var(--space-1) 0 var(--space-5);
		font-size: var(--text-sm);
		color: var(--color-text-muted);
	}

	ul {
		margin: 0;
		padding: 0;
		list-style: none;
		display: grid;
		gap: var(--space-1);
	}

	a {
		display: block;
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-md);
		color: var(--color-text);
		text-decoration: none;
	}

	a:hover {
		background: var(--color-accent-soft);
	}

	a.active {
		background: var(--color-accent-soft);
		color: var(--color-accent);
		font-weight: 600;
	}

	a:focus-visible {
		outline: var(--focus-width) solid var(--color-focus);
		outline-offset: 2px;
	}
</style>
