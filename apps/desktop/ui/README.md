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

Une coquille de navigation : une route par écran de l'inventaire V0.1 (design 0003 §5, hors T1), une barre latérale et un titre par écran ; les contenus arrivent avec les tickets suivants. Les jetons de design sont dans `src/lib/styles/breathly.css` (issus des maquettes). Les polices des maquettes (Bricolage Grotesque, Hanken Grotesk, DM Mono) ne sont **pas encore embarquées** : le texte utilise les polices de repli du système jusqu'à ce que les fichiers WOFF2 et leur licence soient ajoutés.

## Écrans et navigation

`src/lib/navigation/screens.ts` est l'inventaire : identifiant (H1, D1…), chemin, entrée de barre latérale et clé du titre. Chaque écran a un `src/routes/…/+page.svelte` qui n'affiche que `<ScreenPage id="…" />` ; pour ajouter un écran, l'ajouter à `SCREENS`, créer sa route et ses deux titres dans `locales/`. `screens.test.ts` échoue si l'inventaire, les routes et le design 0003 §5 divergent. La barre latérale (`Sidebar.svelte`) liste Accueil, Diagnostic, Espace disque, Historique, Réglages, dans cet ordre ; les écrans de nettoyage (N1 à N5) restent sous « Espace disque ». Au changement d'écran, `ScreenPage` place le focus sur le titre `h1`. Le focus visible et l'activation au clavier de la barre latérale sont à contrôler à la main sur les machines de validation.

## Textes

Les textes sont dans `locales/fr.json` et `locales/en.json` (racine du dépôt) : un objet plat `"clé": "texte"`, ou `{ "one": …, "other": … }` pour un pluriel (paramètre `count`). `{size}` reçoit un nombre d'octets écrit par `Intl` (Go et Mo en français, GB et MB en anglais). `src/lib/i18n/` expose `t('clé', { … })` : la clé est typée d'après `fr.json`, et `en.json` doit avoir les mêmes clés (typage et `keys.test.ts`). `setLocale()` change la langue sans redémarrer.

Langue au démarrage : première entrée de `navigator.languages` prise en charge, sinon l'anglais. Ce que WebView2 (Windows) et WebKitGTK (Linux) exposent dans `navigator.languages` est **non vérifié** : à contrôler sur les machines de validation.

## Thèmes

Trois choix : `light`, `dark`, `system` (`src/lib/theme/`). `system` suit l'OS quand il change (`prefers-color-scheme`) ; le thème appliqué est l'attribut `data-theme` de `<html>`, lu par `breathly.css`. `startThemeSync()` est lancé par la mise en page ; `setThemePreference()` change le choix. Le choix n'est pas encore conservé à la réouverture ni proposé à l'écran : le sélecteur et les réglages arrivent avec le ticket #47. `contrast.test.ts` calcule le contraste des paires de jetons des deux thèmes (4,5:1 pour le texte, 3:1 pour les composants graphiques). La mise en page coupe animations et transitions sous `prefers-reduced-motion`.

## Notes

- Les imports internes passent par l'alias de sous-chemin `#lib/…` (SvelteKit 3), déclaré dans `package.json` : écrire `#lib/brand.js` pour `src/lib/brand.ts`.
- L'outil Tauri (`@tauri-apps/cli`) est dans `apps/desktop/package.json`, car il cherche `src-tauri/` dans le dossier courant et ses sous-dossiers, pas dans ses parents.
- `make check` lance `pnpm lint`, `pnpm typecheck` et `pnpm test` de ce dossier (`tools/stacks/node-api.sh`).
