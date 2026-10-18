# ADR 0011 : Outillage de code (Rust, unsafe, toolchain, interface)

- **Statut** : accepté ; outillage prouvé par sondes négatives le 2026-10-18 (étape 4), voir « Résultats de l'étape 4 » ci-dessous
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Les `CODING.md` du kit sont des gabarits écrits pour une API. Breathly a besoin de règles propres : erreurs, journaux, `unsafe`, version de l'outil, tests et outillage de l'interface. Toute dépendance se demande d'abord (PRD §14).

## Décision

**Rust : trois dépendances acceptées.** `thiserror` (énumérations d'erreurs typées dans les crates de bibliothèque), `anyhow` (aux frontières d'un binaire seulement : application Tauri, service), `tracing` (journaux structurés à liste blanche, SEC9). `tempfile` est accepté en dépendance de développement (tests dans des dossiers temporaires). Constats au 2026-10-18 (crates.io et `cargo info`) : `thiserror` 2.0.21 (publiée le 2026-09-23), `anyhow` 1.0.104 (2026-07-18), `tracing` 0.1.44 (2025-12-18), `tempfile` 3.27.0 (2026-03-11) ; licences MIT ou Apache-2.0 (`tracing` : MIT). Ces versions ne peuvent s'installer qu'après le délai de 7 jours (CONTRIBUTING §8).

**`unsafe` interdit partout, sauf dans `breathly-platform`.** Tous les crates héritent de `[workspace.lints]` avec `unsafe_code = "forbid"`. `breathly-platform` déclare sa **propre** table `[lints]` (copie de celle du workspace, avec `unsafe_code = "deny"`) et n'a un bloc `unsafe` que sous un `#[allow(unsafe_code)]` justifié en commentaire et relu. Vérifié par essai le 2026-10-18 (Rust 1.97.1) : (1) `workspace = true` ne se combine pas avec une clé locale (erreur de lecture du manifeste) ; (2) un crate avec sa propre table `deny` accepte un `allow` sur un bloc ; (3) dans un crate en `forbid`, le même `allow` est refusé (erreur E0453). Un contrôle de CI vérifie que la table de `breathly-platform` reste identique à celle du workspace, hors `unsafe_code` (à écrire à l'étape 5).

**Version de l'outil fixée.** `rust-toolchain.toml` fixe une version précise de Rust stable (1.97.1 à la date de cet ADR), pour que tout le monde et la CI compilent avec le même outil et que les lints de `clippy` ne changent pas sans qu'on l'ait voulu. La mise à jour est une PR volontaire, au moins une fois par trimestre.

**Interface : outillage minimal.**
- Textes : un petit **chargeur typé** qui lit `locales/*` et s'appuie sur `Intl` (pluriels, nombres, unités) ; aucune bibliothèque d'internationalisation. Les clés sont vérifiées par un test (US-002.3).
- Tests : **Vitest** et **Testing Library** pour les composants ; pas de test de bout en bout en V0.1 (essais manuels sur les machines de validation).
- Qualité : **ESLint**, **Prettier** et **svelte-check** (typage). Scripts attendus par le kit dans `apps/desktop/ui/package.json` : `lint` (ESLint et `prettier --check`), `typecheck` (svelte-check), `test` (Vitest).
- Versions et configurations de ces outils : prouvées à l'étape 4 (voir « Résultats de l'étape 4 »).

## Conséquences

- Les `CODING.md` du dépôt racine (Rust) et de `apps/desktop/ui` sont réécrits pour Breathly.
- `Cargo.lints.fragment.toml` est remplacé, à l'initialisation, par `[workspace.lints]` dans le `Cargo.toml` racine.
- La complexité cognitive est contrôlée par `clippy::cognitive_complexity`, qui est dans le groupe `nursery` : un lint du groupe `pedantic` ne l'active pas, il est donc activé nommément dans `[workspace.lints]` ; le seuil est dans `clippy.toml`.
- Le chargeur de textes est du code du projet : il a ses tests et une taille bornée ; s'il grossit, on rouvre le choix par un ADR.

## Résultats de l'étape 4 (2026-10-18)

Preuves, sondes rejouables et versions : `docs/research/2026-10-18-outils-etape-4.md`. Écarts avec le texte ci-dessus, tous constatés par essai :

- **Le gabarit officiel de Vitest ne convient pas.** Le complément Vitest de `sv create` utilise un navigateur Playwright (téléchargement d'un navigateur entier). Remplacé par **jsdom** et `@testing-library/svelte`, ce que l'ADR prévoyait (Vitest et Testing Library). Aucun navigateur à installer pour `make check`.
- **Lints Rust ajoutés** aux lints de départ : `excessive_nesting`, `cognitive_complexity`, `wildcard_enum_match_arm`, `allow_attributes_without_reason` (tous hors des groupes `all` et `pedantic`, donc à nommer un par un) ; seuils dans `clippy.toml`.
- **`cargo-deny` est ajouté** (outil de développement, pas une dépendance du produit) : il porte la garde « aucune bibliothèque réseau » (SEC1) et le **sens des dépendances entre crates**, que le compilateur n'impose pas. Il ne contrôle que les cibles Windows et Linux : sans cela, `tauri` tire `reqwest` pour Android et iOS (constaté avec tauri 2.12.4).
- **`tools/check-rust.py`** (script du projet, sans dépendance) couvre ce que ni `rustc` ni `clippy` ne contrôlent : table `[lints]` de chaque crate, table de `breathly-platform` identique hors `unsafe_code`, aucun `#[ignore]`, aucun `cfg` selon l'OS hors de `breathly-platform`.
