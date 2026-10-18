# apps/desktop/ui : interface Svelte 5 (SvelteKit en mode SPA)

Interface de Breathly. Elle affiche et déclenche ; toute règle métier vit dans `crates/` (voir `CODING.md` de ce dossier, ADR 0010 et 0011).

## Prérequis

Node.js 24, pnpm 11 (version lue dans le `package.json` de la racine) et, pour l'application complète, les paquets de développement de Tauri (`docs/guides/demarrage-developpeur.md` §2).

## En 5 commandes (depuis la racine du dépôt)

```bash
pnpm install                          # installe l'interface et l'outil Tauri (espace de travail pnpm)
pnpm --dir apps/desktop/ui dev        # l'interface seule, dans un navigateur (http://127.0.0.1:5173)
pnpm --dir apps/desktop dev           # l'application complète : fenêtre Tauri + interface
pnpm --dir apps/desktop build         # installeurs (deb, rpm, AppImage sous Linux) dans target/release/bundle/
make check                            # contrôles avant PR (lint, typage, tests, et le cœur Rust)
```

## Contenu actuel

Une coquille : une page qui affiche le nom du produit avec les jetons de design (`src/lib/styles/breathly.css`, issu des maquettes). Les écrans arrivent avec les tickets de la V0.1. Les polices des maquettes (Bricolage Grotesque, Hanken Grotesk, DM Mono) ne sont **pas encore embarquées** : le texte utilise les polices de repli du système jusqu'à ce que les fichiers WOFF2 et leur licence soient ajoutés.

## Notes

- Les imports internes passent par l'alias de sous-chemin `#lib/…` (SvelteKit 3), déclaré dans `package.json` : écrire `#lib/brand.js` pour `src/lib/brand.ts`.
- L'outil Tauri (`@tauri-apps/cli`) est dans `apps/desktop/package.json`, car il cherche `src-tauri/` dans le dossier courant et ses sous-dossiers, pas dans ses parents.
- `make check` lance `pnpm lint`, `pnpm typecheck` et `pnpm test` de ce dossier (`tools/stacks/node-api.sh`).
