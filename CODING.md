# Écrire le code Rust de Breathly

Règles d'écriture du cœur Rust (`crates/`) et de la partie Rust de l'application (`apps/desktop/src-tauri`). Elles complètent `AGENTS.md` (invariants), `docs/policies/securite.md` (SEC1 à SEC11), les ADR 0002, 0010 et 0011 et le design 0002. **[machine]** casse `make check` ; **[relecture]** engage le relecteur.

> Chaque garde marquée [machine] a été prouvée le 2026-10-18 (étape 4) : elle fait échouer `make check` sur une violation volontaire (sonde négative). Preuves, versions et sondes rejouables : `docs/research/2026-10-18-outils-etape-4.md`.

Mot-guide : **le type porte la règle**. Un état impossible ne doit pas se pouvoir représenter, et une suppression ne doit pas se pouvoir écrire sans passer par les garde-fous.

## 1. Où va quoi

- Workspace Cargo à la racine (ADR 0010). Un crate par module : `breathly-platform`, `-rules`, `-diagnose`, `-disk` en V0.1. Le sens des dépendances est celui du PRD §5 et du design 0002 §3 ; il est déclaré dans les `Cargo.toml` et **contrôlé par `cargo-deny`** (`deny.toml`, option `wrappers` : seuls les crates listés peuvent dépendre d'un crate) **[machine]**. Le compilateur, lui, n'interdit qu'un cycle : ne pas s'y fier. Les crates n'importent jamais `apps/` ; `apps/desktop/src-tauri` importe les crates.
- Visibilité minimale : `pub(crate)` par défaut ; seule l'API listée au design 0002 §2 est `pub`.
- Tout ce qui dépend de l'OS vit derrière un trait de `breathly-platform` ; **aucun `cfg(target_os)`, `cfg(windows)` ni `cfg(unix)` ailleurs** **[machine]** (`tools/check-rust.py`).
- Un crate de bibliothèque n'affiche aucun texte à l'utilisateur : il renvoie des codes et des clés de traduction (`locales/`).

## 2. Écrire une fonction

Fonction ≤ 40 lignes **[machine]** (`clippy::too_many_lines`), imbrication ≤ 3 (retours anticipés et `?`) **[machine]** (`clippy::excessive_nesting`), complexité cognitive ≤ 10 (`clippy::cognitive_complexity`, groupe `nursery` : activé lint par lint dans `[workspace.lints]`, seuil dans `clippy.toml`) **[machine]**, ≤ 3 paramètres (au-delà, une structure) **[machine]**, fichier ≤ 250 à 300 lignes **[relecture]** (clippy n'a pas de lint pour la taille d'un fichier). Nommage du `GLOSSARY.md` : `Item`/élément, `RiskLevel`, `Rule`, `Guardrail`. Pas de booléen muet : une `enum` à deux variantes.

## 3. Types

- Newtypes pour les valeurs métier : `RuleId`, `ItemId`, `ByteSize`, `HistoryEntryId`. Niveau de risque, état d'une tâche, état d'une entrée d'historique : des `enum` avec `match` **sans** branche `_` (design 0001) **[machine]** (`clippy::wildcard_enum_match_arm`).
- **Une suppression n'accepte qu'un `VerifiedItem`**, type que seul le module des garde-fous peut construire après avoir revérifié le chemin (SEC2). Écrire une fonction de suppression qui prend un `PathBuf` brut est un défaut bloquant.
- Constructeurs qui valident (`TryFrom`) ; immutabilité par défaut ; clonage justifié.
- `unwrap()` interdit hors tests (`clippy::unwrap_used`, autorisé dans les tests par `clippy.toml`) **[machine]** ; `expect()` seulement avec un message qui dit l'invariant ; pas de `panic!` pour une erreur attendue.

## 4. `unsafe`

`unsafe_code = "forbid"` dans tous les crates via `[workspace.lints]`. Seul `breathly-platform` peut avoir du `unsafe`, avec sa propre table `[lints]` (ADR 0011) et un `#[allow(unsafe_code, reason = "…")]` par bloc (tout `#[allow]` exige un `reason`, `clippy::allow_attributes_without_reason` **[machine]**), un commentaire `// SAFETY:` qui prouve l'invariant, et une relecture explicite **[relecture]**. `unsafe` hors de `breathly-platform` casse `make check` **[machine]**, et `tools/check-rust.py` vérifie que la table `[lints]` de `breathly-platform` reste celle du workspace hors `unsafe_code` **[machine]**.

## 5. Erreurs et journaux

- Bibliothèques : `thiserror`, une `enum` d'erreurs par crate, variantes métier (`GuardrailRefused`, `HistoryUnwritable`…). `anyhow` seulement aux frontières d'un binaire. Un échec métier attendu est une variante, pas un `panic`.
- Chaque erreur porte un **code stable** et une **clé de traduction** ; l'interface choisit le texte (US-002.3).
- On ajoute du contexte avant de remonter ; jamais d'erreur ignorée (`let _ =` justifié par un commentaire).
- Journaux `tracing` : **liste blanche** (SEC9). Jamais un chemin complet, le contenu d'un fichier, un titre de fenêtre ou une URL ; des identifiants et des codes seulement **[relecture + test]**.

## 6. Sécurité (résumé de SEC1 à SEC11)

- **Aucune dépendance réseau** dans le cœur ni l'interface (SEC1) **[machine]** (`deny.toml`, liste de crates interdits, cibles Windows et Linux) ; nouvelle dépendance = demande préalable (PRD §14) et ADR si elle structure.
- **Aucun lancement de programme externe** en V0.1 : `std::process::Command` n'apparaît pas hors de `breathly-platform` ; le champ `command` d'une règle n'est jamais exécuté (SEC6) **[machine]** (`clippy::disallowed_methods` et `disallowed_types` dans `clippy.toml` ; `breathly-platform` y déroge par un `#[allow(…, reason)]` relu).
- Suppression : seulement par une règle, garde-fous revérifiés **pour chaque élément** au moment d'agir, liens symboliques non suivis, historique « pending » écrit avant l'action (SEC2, SEC4). L'interface n'envoie jamais un chemin (SEC3).
- Fichiers lus (réglages, exclusion, historique) : validés, taille bornée, jamais de valeur corrigée en silence (SEC5).
- Élévation (V0.4) : jamais de commande libre vers l'assistant (SEC7).
- `cargo audit` en CI et en `make ci-local` **[machine]** ; versions verrouillées (`--locked`) **[machine]**.

## 7. Tests

- `cargo test` : tests unitaires du domaine sans I/O ; tests d'intégration dans un **dossier temporaire** (`tempfile`). **Aucun test ne touche au vrai dossier personnel ni à la vraie corbeille** : le faux `TrashProvider` est obligatoire (PRD §14).
- Un test **par garde-fou** de S5 (SEC2), un test de lien symbolique et de jonction, un test d'historique non inscriptible, un test par état de la machine à états (design 0001). La liste des tests de sécurité est dans `docs/policies/modele-de-menaces.md` (TS-01 à TS-16).
- L'heure passe par le trait `Clock`. Aucun test ignoré : `#[ignore]` est refusé par `tools/check-rust.py` **[machine]** ; un test supprimé ou affaibli pour passer au vert est refusé en relecture **[relecture]**.

## 8. Code verbeux : ce qu'on retire

Commentaire qui répète le code, trait à une seule implémentation sans besoin de test ou de variation d'OS, générique « au cas où » (YAGNI), règle métier écrite deux fois, crate vide pour une version qui n'a pas commencé (ADR 0010).

## 9. Définition de « terminé »

- [ ] `make check` vert : `cargo fmt --check`, `clippy -D warnings`, `cargo deny`, `tools/check-rust.py`, `cargo test`, scan de secrets ; `make audit` (`cargo audit`) vert
- [ ] Aucun `unwrap()` hors tests, aucun `unsafe` hors `breathly-platform`
- [ ] Chaque échec métier attendu est une variante d'erreur avec un code et une clé de traduction
- [ ] Une suppression passe par un `VerifiedItem` ; aucun chemin brut venu de l'interface
- [ ] Un test échoue sans le changement ; aucun test ne touche au vrai disque
- [ ] Les textes affichés sont dans `locales/`, pas dans le Rust
