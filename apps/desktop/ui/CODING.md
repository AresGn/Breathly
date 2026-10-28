# Écrire le code de l'interface (Svelte 5 + TypeScript)

Règles d'écriture de `apps/desktop/ui` (SvelteKit en mode SPA, ADR 0010). Elles complètent `AGENTS.md`, `docs/policies/securite.md` (SEC1 à SEC11), l'ADR 0011 et les maquettes de `docs/design/maquettes/`. **[machine]** casse `make check` ; **[relecture]** engage le relecteur.

> Chaque garde marquée [machine] a été prouvée le 2026-10-18 (étape 4) : elle fait échouer `make check` sur une violation volontaire (sonde négative). Preuves et versions : `docs/research/2026-10-18-outils-etape-4.md`. Les configurations éprouvées (ESLint, Vite/Vitest, Prettier, test des jetons) sont dans `docs/research/outils-etape-4/interface/` : l'initialisation (étape 5) les recopie. Une garde marquée « à écrire » n'existe pas encore : elle s'écrit avec le code qu'elle protège.

Mot-guide : **l'interface affiche et déclenche, elle ne décide pas.** Toute règle métier vit dans les crates Rust.

## 1. Où va quoi

| Dossier | Contenu |
|---|---|
| `src/routes/` | une route par écran (SvelteKit) ; `+layout.ts` avec `export const ssr = false;` |
| `src/lib/components/` | composants de présentation purs : propriétés en entrée, événements en sortie, aucune logique métier |
| `src/lib/api/` | **le seul endroit** qui appelle les commandes Tauri (`invoke`), avec des types |
| `src/lib/generated/` | types générés depuis Rust, jamais écrits à la main **[machine, à écrire avec la génération des types]** (la CI vérifiera qu'ils sont à jour) |
| `src/lib/state/` | états partagés en `.svelte.ts` (runes) |
| `src/lib/i18n/` | chargeur de textes typé ; les textes sont dans `locales/` |
| `src/lib/styles/` | jetons de design (`breathly.css` des maquettes) |

## 2. Composants

Un composant = un fichier, ≤ 250 lignes **[machine]** (`max-lines`) ; fonction ≤ 40 lignes, complexité ≤ 10, imbrication ≤ 3, ≤ 3 paramètres **[machine]** (ESLint). Présentation pure : il reçoit des données et émet des événements. Pas de `invoke` dans un composant : il passe par `src/lib/api/` **[machine]** (`no-restricted-imports` sur `@tauri-apps/*` hors de `src/lib/api/`). Pas de calcul de chemin, d'espace libéré ou de niveau de risque : ces valeurs viennent du cœur.

## 3. État

Svelte 5 : `$state` et `$derived` dans des modules `.svelte.ts`, **aucune bibliothèque d'état** de plus. La vérité est dans le cœur Rust : l'interface garde une copie de lecture. L'état d'une tâche est une union discriminée qui reprend le design 0001 (`idle`, `running`, `done`, `cancelled`, `failed`), avec un `switch` **exhaustif** terminé par `assertNever` **[machine]** (`@typescript-eslint/switch-exhaustiveness-check` : un cas manquant ou un `default` seul échoue).

## 4. Types

TypeScript strict **[machine]** (`svelte-check`) ; `any` interdit (`@typescript-eslint/no-explicit-any`) **[machine]** ; `@ts-ignore` interdit et `@ts-expect-error` seulement avec une raison de 10 caractères au moins (`ban-ts-comment`) **[machine]**. Les données qui arrivent de Rust sont typées par les types générés, jamais par un `as` **[machine]** (`consistent-type-assertions: never`).

## 5. Textes

Aucune chaîne affichée n'est écrite dans un composant : tout passe par une **clé typée** du chargeur (`t('home.analyze')`). Nombres, pluriels et unités par `Intl` (Go/Mo en français, GB/MB en anglais). Un test vérifie que chaque clé existe en FR et en EN et qu'aucune clé n'est orpheline (US-002.3) **[machine]** (`src/lib/i18n/cles.test.ts`). Une chaîne en dur dans un composant n'est pas détectée par la machine : **[relecture]**. Vocabulaire du `GLOSSARY.md` : « logiciel », jamais « application ».

## 6. Styles et thèmes

Uniquement les **jetons** de `breathly.css` : aucune couleur, taille ou ombre en dur dans un composant : les **couleurs** sont contrôlées par un test Vitest qui lit les sources (`jetons.test.ts`) **[machine]** ; les tailles et les ombres restent **[relecture]**. Les deux thèmes (`data-theme="light"` et `"dark"`) sont toujours testés. Aucune animation lourde (WebKitGTK) ; l'anneau de l'écran H2 tourne seulement si `prefers-reduced-motion` n'est pas demandé.

## 7. Accessibilité (WCAG 2.2 AA)

Chaque action est atteignable au clavier avec un focus visible ; chaque contrôle a un nom accessible ; le texte reste lisible à 200 % ; **le niveau de risque n'est jamais indiqué par la couleur seule** (icône, libellé et couleur) **[relecture + test]**.

## 8. Sécurité

- Aucun contenu distant : pas de police, d'image, de script ni de feuille de style chargés depuis Internet (SEC1) ; polices et icônes embarquées.
- Pas de `{@html ...}` (`svelte/no-at-html-tags`) ni d'`eval` (`no-eval`, `no-implied-eval`) **[machine]**.
- **L'interface n'envoie jamais un chemin** à une commande : seulement des identifiants d'éléments et un jeton de confirmation (SEC3).
- Aucun `console.*` (SEC9) **[machine]** (`no-console`) : pas de journal du tout côté interface.
- Toute nouvelle commande Tauri est ajoutée aux capacités de la fenêtre principale seulement, avec le cœur.

## 9. Tests

Vitest et Testing Library : un test par état affiché (chargement, vide, erreur, « non disponible », mode simple et avancé), un test des clés de traduction, un test d'accessibilité de base (noms des contrôles). Pas de test de bout en bout en V0.1 : essais manuels sur les machines de validation (PRD §13). Aucun test ignoré (`it.skip`, `.only`, `.todo`, `.fails` refusés par ESLint **[machine]**), supprimé ou affaibli **[relecture]**. Un test sans assertion échoue (`expect.requireAssertions`) **[machine]**.

## 10. Imports et scripts

Imports internes par l'alias de sous-chemin `#lib/…` (SvelteKit 3, déclaré dans `package.json`), avec l'extension `.js` pour un fichier TypeScript : `#lib/brand.js`. L'alias `$lib` n'est pas utilisé.

## 11. Scripts attendus par le kit

`pnpm lint` (`prettier --check` puis ESLint), `pnpm typecheck` (svelte-check, avertissements bloquants), `pnpm test` (Vitest, jsdom) et `pnpm fix` (corrige la mise en forme).

## 12. Définition de « terminé »

- [ ] `make check` vert (`pnpm lint`, `pnpm typecheck`, `pnpm test`)
- [ ] Aucun `any`, aucune couleur en dur, aucune chaîne affichée en dur
- [ ] Les deux thèmes et les deux langues fonctionnent
- [ ] Mode simple : aucun chemin ni nom de processus brut sans clic sur « Détails »
- [ ] Le composant correspond à la maquette (`docs/design/maquettes/`)
- [ ] Un test échoue sans le changement
