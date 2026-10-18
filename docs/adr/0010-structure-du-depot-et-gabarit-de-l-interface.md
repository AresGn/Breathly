# ADR 0010 : Structure du dépôt et gabarit de l'interface

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Breathly réunit un cœur Rust (plusieurs modules), une interface de bureau (Tauri v2 + Svelte 5, ADR 0001), une extension de navigateur (V0.3), des fichiers de données (règles, définitions de navigateurs, traductions) et des binaires secondaires (service, hôte, assistant d'élévation). Le PRD §11 donnait une structure « prévue » ; il fallait la figer avant d'initialiser quoi que ce soit, ainsi que le gabarit de l'interface (l'ADR 0001 dit Svelte 5 sans préciser).

## Options

**Disposition** : (A) workspace Cargo à la racine avec `crates/` et `apps/desktop/` ; (B) exécutables dans `apps/`, bibliothèques seules dans `crates/` ; (C) tout le Rust sous `core/`.
**Gabarit de l'interface** : (1) SvelteKit en mode SPA ; (2) Svelte + Vite avec un routeur léger ; (3) une page et une navigation par état.

## Décision

**Disposition A**, celle du PRD §11, de l'ADR 0002 et d'`AGENTS.md`. Les commandes `cargo` se lancent depuis la racine.

```
breathly/
├── Cargo.toml                 workspace Rust (membres : crates/*, apps/desktop/src-tauri)
├── pnpm-workspace.yaml        workspace pnpm (apps/desktop/ui ; extension/ en V0.3)
├── crates/
│   ├── breathly-platform/     module platform
│   ├── breathly-rules/        module rules
│   ├── breathly-diagnose/     module diagnose
│   └── breathly-disk/         modules disk-scan et disk-clean
├── apps/desktop/
│   ├── src-tauri/             crate Tauri, membre du workspace Cargo (module app-ui, côté Rust)
│   └── ui/                    interface Svelte 5 (SvelteKit en mode SPA)
├── rules/  browsers/  locales/    données TOML et traductions
├── docs/  tools/  .github/
└── extension/                 créé en V0.3
```

**Gabarit SvelteKit en mode SPA (option 1).** C'est la voie documentée par Tauri : `@sveltejs/adapter-static` avec `fallback: 'index.html'`, rendu serveur désactivé par `export const ssr = false;` dans `src/routes/+layout.ts`, `frontendDist` pointant vers le dossier construit. Le routage par fichiers sert aux 20 écrans sans routeur de plus. Sources lues le 2026-10-18 : https://v2.tauri.app/start/frontend/sveltekit/ ; et le gabarit officiel de `create-tauri-app` 4.8.0 (modèle `svelte-ts`) déclare `svelte` ^5.56.3 et `@sveltejs/adapter-static`, ce qui confirme que Svelte 5 et Tauri v2 vont ensemble au niveau du gabarit.

**Règle de croissance.** Seul ce que la version en cours utilise existe. En V0.1 : le workspace, les quatre crates ci-dessus et `apps/desktop`. Les crates `-daemon`, `-guard`, `-watch`, `-nmhost`, `-startup`, `-tune`, `-elevated` et le dossier `extension/` sont créés quand leur version démarre, avec leur ticket : pas de crate vide qui mente sur l'avancement. Le sens des dépendances entre crates reste celui du PRD §5 (design 0002).

**Où va quoi.**

| Je cherche… | Dossier |
|---|---|
| une règle métier, un garde-fou, une mesure système | un crate de `crates/` |
| une commande Tauri (frontière entre l'interface et le cœur) | `apps/desktop/src-tauri/` |
| un écran, un composant, un style, un texte affiché | `apps/desktop/ui/` (textes dans `locales/`) |
| une règle de nettoyage ou une définition de navigateur | `rules/` et `browsers/` (TOML, aucune ligne de Rust) |
| une traduction | `locales/` |
| une maquette | `docs/design/maquettes/` |

## Conséquences

- `.kit.conf` déclare deux applications : `.` (pile `rust`) et `apps/desktop/ui` (pile `node-api`, qui exécute `lint`, `typecheck` et `test` de son `package.json`). L'extension s'y ajoutera en V0.3 (`extension:node-api`).
- Le gabarit SvelteKit apporte un script `check` (svelte-check) et non `typecheck` : l'initialisation ajoute les scripts attendus par le kit.
- L'outil Tauri (`@tauri-apps/cli`) vit dans `apps/desktop/package.json` : il cherche `src-tauri/` dans le dossier courant et ses sous-dossiers, pas dans les parents (essai du 2026-10-18). L'espace de travail pnpm compte donc deux paquets, `apps/desktop` et `apps/desktop/ui`.
- L'interface et le cœur dépendent de types partagés générés depuis Rust (design 0002 §6) ; l'emplacement des types générés est `apps/desktop/ui/src/lib/generated/` (non écrit à la main).
- `apps/desktop/src-tauri` est un membre du workspace Cargo : `cargo clippy --workspace` couvre aussi l'application. Son identifiant d'application (`identifier` de `tauri.conf.json`) reste à décider à l'initialisation.
- Les crates de bibliothèque n'importent jamais `apps/` ; `apps/` importe `crates/`.
- Si l'ensemble grossit au point que la racine devienne illisible, la disposition B (exécutables dans `apps/`) reste ouverte par un nouvel ADR.

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Définit seulement l'emplacement des fichiers et l'ordre de création.
