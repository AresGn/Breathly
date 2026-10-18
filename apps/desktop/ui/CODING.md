# Écrire le code de l'interface (Svelte 5 + TypeScript)

Règles d'écriture de `apps/desktop/ui` (SvelteKit en mode SPA, ADR 0010). Elles complètent `AGENTS.md`, `docs/policies/securite.md` (SEC1 à SEC11), l'ADR 0011 et les maquettes de `docs/design/maquettes/`. **[machine]** casse `make check` ; **[relecture]** engage le relecteur.

> Les outils de cette page (ESLint, Prettier, svelte-check, Vitest) sont décidés par l'ADR 0011 mais **restent à prouver à l'étape 4** : chaque garde marquée [machine] doit échouer sur une violation volontaire avant de compter.

Mot-guide : **l'interface affiche et déclenche, elle ne décide pas.** Toute règle métier vit dans les crates Rust.

## 1. Où va quoi

| Dossier | Contenu |
|---|---|
| `src/routes/` | une route par écran (SvelteKit) ; `+layout.ts` avec `export const ssr = false;` |
| `src/lib/components/` | composants de présentation purs : propriétés en entrée, événements en sortie, aucune logique métier |
| `src/lib/api/` | **le seul endroit** qui appelle les commandes Tauri (`invoke`), avec des types |
| `src/lib/generated/` | types générés depuis Rust, jamais écrits à la main **[machine]** (la CI vérifie qu'ils sont à jour) |
| `src/lib/state/` | états partagés en `.svelte.ts` (runes) |
| `src/lib/i18n/` | chargeur de textes typé ; les textes sont dans `locales/` |
| `src/lib/styles/` | jetons de design (`breathly.css` des maquettes) |

## 2. Composants

Un composant = un fichier, ≤ 250 lignes. Présentation pure : il reçoit des données et émet des événements. Pas de `invoke` dans un composant : il passe par `src/lib/api/`. Pas de calcul de chemin, d'espace libéré ou de niveau de risque : ces valeurs viennent du cœur.

## 3. État

Svelte 5 : `$state` et `$derived` dans des modules `.svelte.ts`, **aucune bibliothèque d'état** de plus. La vérité est dans le cœur Rust : l'interface garde une copie de lecture. L'état d'une tâche est une union discriminée qui reprend le design 0001 (`idle`, `running`, `done`, `cancelled`, `failed`), avec un `switch` **exhaustif** terminé par `assertNever` **[machine]**.

## 4. Types

TypeScript strict **[machine]** ; `any` interdit (`@typescript-eslint/no-explicit-any`) **[machine]** ; `@ts-ignore` interdit sans raison écrite sur la ligne. Les données qui arrivent de Rust sont typées par les types générés, jamais par un `as`.

## 5. Textes

Aucune chaîne affichée n'est écrite dans un composant : tout passe par une **clé typée** du chargeur (`t('home.analyze')`). Nombres, pluriels et unités par `Intl` (Go/Mo en français, GB/MB en anglais). Un test vérifie que chaque clé existe en FR et en EN et qu'aucune clé n'est orpheline (US-002.3) **[machine]**. Vocabulaire du `GLOSSARY.md` : « logiciel », jamais « application ».

## 6. Styles et thèmes

Uniquement les **jetons** de `breathly.css` : aucune couleur, taille ou ombre en dur dans un composant **[relecture, règle ESLint à prouver]**. Les deux thèmes (`data-theme="light"` et `"dark"`) sont toujours testés. Aucune animation lourde (WebKitGTK) ; l'anneau de l'écran H2 tourne seulement si `prefers-reduced-motion` n'est pas demandé.

## 7. Accessibilité (WCAG 2.2 AA)

Chaque action est atteignable au clavier avec un focus visible ; chaque contrôle a un nom accessible ; le texte reste lisible à 200 % ; **le niveau de risque n'est jamais indiqué par la couleur seule** (icône, libellé et couleur) **[relecture + test]**.

## 8. Sécurité

- Aucun contenu distant : pas de police, d'image, de script ni de feuille de style chargés depuis Internet (SEC1) ; polices et icônes embarquées.
- Pas de `{@html ...}` avec une donnée dynamique, pas d'`eval`.
- **L'interface n'envoie jamais un chemin** à une commande : seulement des identifiants d'éléments et un jeton de confirmation (SEC3).
- Aucun titre de fenêtre, chemin ni URL dans `console.*` (SEC9).
- Toute nouvelle commande Tauri est ajoutée aux capacités de la fenêtre principale seulement, avec le cœur.

## 9. Tests

Vitest et Testing Library : un test par état affiché (chargement, vide, erreur, « non disponible », mode simple et avancé), un test des clés de traduction, un test d'accessibilité de base (noms des contrôles). Pas de test de bout en bout en V0.1 : essais manuels sur les machines de validation (PRD §13). Aucun test ignoré, supprimé ou affaibli.

## 10. Scripts attendus par le kit

`pnpm lint` (ESLint et `prettier --check`), `pnpm typecheck` (svelte-check), `pnpm test` (Vitest).

## 11. Définition de « terminé »

- [ ] `make check` vert (`pnpm lint`, `pnpm typecheck`, `pnpm test`)
- [ ] Aucun `any`, aucune couleur en dur, aucune chaîne affichée en dur
- [ ] Les deux thèmes et les deux langues fonctionnent
- [ ] Mode simple : aucun chemin ni nom de processus brut sans clic sur « Détails »
- [ ] Le composant correspond à la maquette (`docs/design/maquettes/`)
- [ ] Un test échoue sans le changement
