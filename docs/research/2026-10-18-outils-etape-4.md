# Étape 4 : outils prouvés par sondes négatives

- **Date** : 2026-10-18
- **Question** : chaque garde que `AGENTS.md`, les `CODING.md` et `securite.md` marquent « machine » casse-t-elle vraiment `make check` quand on la viole ?
- **Alimente** : ADR 0011 (outillage), ADR 0002 (sens des dépendances), SEC1 et SEC10 (`docs/policies/securite.md`), `CODING.md` (cœur) et `apps/desktop/ui/CODING.md`.
- **Poste** : Linux Mint, Rust 1.97.1, Node 24.16.0, pnpm 11.5.3, Python 3.11.2.

## Méthode

Une sonde négative est une violation volontaire. Elle ne compte que si l'outil la refuse **pour la bonne raison** : chaque sonde cherche le nom du lint ou de la règle dans la sortie, car un échec pour une autre raison ne prouve rien (les premières sondes de cette étape échouaient toutes pour une raison étrangère : code collé après un module de test, `deref_addrof`, `# Panics` manquant, ligne non formatée qui arrête `cargo fmt` avant clippy). Chaque sonde a aussi sa contrepartie conforme, qui doit passer : une garde qui refuse tout ne prouve rien non plus. Deuxième piège rencontré : restaurer un fichier avec son ancienne date de modification fait réutiliser à `cargo` un binaire périmé, et la sonde conforme suivante échouait à tort (le test d'une sonde précédente tournait encore) ; les scripts restaurent maintenant avec la date du jour.

Deux niveaux, rejouables avec les scripts de `outils-etape-4/sondes/` (chemins de la machine d'essai à adapter, voir plus bas) :

1. **Par outil**, sur un workspace Cargo (`probe-rust`, 4 crates V0.1 aux dépendances du PRD §5) et sur un projet SvelteKit (`probe-ui`) créés dans un dossier temporaire, hors du dépôt.
2. **Par `make check`**, sur une copie du dépôt où les deux projets sont placés aux emplacements de l'ADR 0010 (`crates/breathly-*`, `apps/desktop/ui`) : c'est ce niveau qui prouve le câblage de `tools/stacks/*.sh`.

## Outils retenus et versions

| Outil | Version | Rôle | Source de la version |
|---|---|---|---|
| Rust, `rustfmt`, `clippy` | 1.97.1 | format, lints | `rust-toolchain.toml` (étape 5) |
| `cargo-audit` | 0.22.2 (2026-06-05) | avis de sécurité (`make audit`) | `cargo info`, crates.io |
| `cargo-deny` | 0.20.2 | bibliothèques réseau interdites, sens des dépendances, sources | `cargo info` |
| ESLint / `typescript-eslint` / `eslint-plugin-svelte` | 10.12.0 / 8.71.1 / 3.23.1 | lint de l'interface | installés par pnpm avec `minimumReleaseAge` |
| Prettier / `prettier-plugin-svelte` | 3.9.10 / 4.1.1 | mise en forme | idem |
| `svelte-check` / TypeScript | 4.7.6 / 6.0.3 | typage (`--fail-on-warnings`) | idem |
| Vitest / jsdom / Testing Library (`svelte`, `jest-dom`) | 4.1.11 / 30.1.2 / 5.4.2, 7.0.1 | tests | idem |
| SvelteKit / Svelte / Vite / `adapter-static` | 3.0.1 / 5.57.2 / 8.3.4 / 4.0.0 | gabarit (`sv create` 1.1.1) | idem |

Les versions réellement installées sont celles que `pnpm` a retenues sous la règle des 7 jours : `@sveltejs/kit` 3.0.1 alors que 3.1.0 existait (2026-10-14), `eslint` 10.12.0 alors que 10.13.0 existait (2026-10-16), etc.

## Résultats par outil

### Rust

| Garde (source) | Sonde | Résultat |
|---|---|---|
| `cargo fmt --check` (AGENTS) | fichier mal formaté | refusé |
| `clippy` `all` en `deny` | `return` inutile | refusé (`needless_return`) |
| `clippy` `pedantic` en `warn` + `-D warnings` | fonction publique sans `#[must_use]` | refusé (`must_use_candidate`) |
| `unwrap_used` (CODING §3) | `unwrap()` hors test | refusé |
| idem, dans un test | `unwrap()` dans `#[cfg(test)]` | **accepté** (voulu : `allow-unwrap-in-tests`) |
| `expect_used` | `expect()` hors test | refusé |
| `cognitive_complexity` ≤ 10 (nursery) | fonction à complexité 11 et plus | refusé ; **une fonction de complexité exactement 10 passe** (mon premier essai valait 10 et n'était pas détecté) |
| `too_many_lines` ≤ 40 | fonction de 47 lignes | refusé |
| `too_many_arguments` ≤ 3 | 4 paramètres | refusé |
| `excessive_nesting` ≤ 3 (nursery) | 4 `if` imbriqués | refusé ; 3 passent (le corps de la fonction compte pour un niveau : seuil `clippy.toml` à 4) |
| `unsafe_code = "forbid"` | `unsafe` dans `breathly-rules` | refusé |
| `breathly-platform` en `deny` | `unsafe` nu | refusé |
| idem avec `#[allow(unsafe_code, reason = …)]` | `unsafe` justifié | **accepté** (voulu) |
| `forbid` ne tolère pas `allow` | `#[allow(unsafe_code)]` dans `breathly-rules` | refusé (E0453) |
| `match` sans `_` (AGENTS) | `match` avec `_ =>` sur un `enum` | **non détecté par défaut** : il faut nommer `wildcard_enum_match_arm` ; ensuite refusé, et le `match` exhaustif passe |
| `#[allow]` toujours justifié (CONTRIBUTING règle 6) | `#[allow(…)]` sans `reason` | **non détecté par défaut** : `allow_attributes_without_reason` ajouté ; ensuite refusé |
| aucun `Command` hors platform (SEC6) | `std::process::Command::new` | **non détecté par défaut** : `disallowed-methods` et `disallowed-types` dans `clippy.toml` ; ensuite refusé, accepté dans `breathly-platform` avec `#[allow(…, reason)]` |
| aucun test ignoré (CODING §7) | `#[ignore = "raison"]` | **non détecté** (clippy n'exige qu'une raison) : `tools/check-rust.py` |

### Garde-fous de structure (`cargo-deny`, `tools/check-rust.py`)

| Garde | Sonde | Résultat |
|---|---|---|
| sens des dépendances (PRD §5) | `platform` dépend de `rules` ; `rules` dépend de `platform` ; `diagnose` dépend de `disk` | refusé (« is explicitly banned ») |
| idem | dépendances légitimes (`diagnose`→`platform`, `disk`→`platform` et `rules`) | accepté |
| SEC1 | `ureq` en dépendance ; `ureq` en dépendance de développement | refusé |
| SEC1 sur l'arbre réel de `tauri` 2.12.4 (Linux, Windows) | `tauri` seul, caractéristiques `wry` et `compression` | accepté **à condition de limiter les cibles** : sans cela `reqwest`, `hyper` et `hyper-util` sont dans l'arbre (cibles Android et iOS) |
| dépôt non officiel | dépendance `git` | refusé (`unknown-git`) |
| versions `*` | `smallvec = "*"` | refusé |
| table `[lints]` | crate sans `[lints] workspace = true` ; table de `breathly-platform` modifiée (autre lint, ou `unsafe_code` en `forbid`) | refusé par `check-rust.py` |
| aucun `cfg` selon l'OS hors platform | `#[cfg(target_os = …)]` dans `breathly-rules` | refusé ; accepté dans `breathly-platform` |
| aucun `#[ignore]` | `#[ignore]` nu | refusé |
| avis de sécurité (SEC10) | `smallvec =1.6.0` (RUSTSEC-2021-0003) | `cargo audit` refuse |
| versions verrouillées | `--locked` après modification du manifeste | refusé |

### Interface

| Garde | Sonde | Résultat |
|---|---|---|
| Prettier | fichier mal formaté | refusé (`pnpm lint`) |
| `no-explicit-any` | `any` | refusé |
| `ban-ts-comment` | `@ts-ignore` ; `@ts-expect-error` sans raison | refusé ; avec raison de 10 caractères : accepté |
| `consistent-type-assertions: never` | `JSON.parse(…) as number` | refusé |
| `no-console` (SEC9) | `console.log` | refusé |
| `svelte/no-at-html-tags` | `{@html …}` | refusé |
| taille (AGENTS) | complexité 11, imbrication 4, 4 paramètres, fonction de 47 lignes, fichier de 260 lignes | refusé (`complexity`, `max-depth`, `max-params`, `max-lines-per-function`, `max-lines`) |
| états en union fermée | `switch` sans un cas ; `switch` avec seulement `default` | refusé (`switch-exhaustiveness-check`) ; avec tous les cas et `assertNever` : accepté |
| `invoke` seulement dans `src/lib/api/` | import de `@tauri-apps/*` ailleurs | refusé (`no-restricted-imports`) ; dans `src/lib/api/` : accepté |
| `svelte-check` | type incompatible ; `<img>` sans `alt` (accessibilité) ; propriété inconnue d'un composant | refusé (avertissements bloquants avec `--fail-on-warnings`) |
| Vitest | assertion fausse ; test sans assertion (`expect.requireAssertions`) | refusé |
| aucun test sauté | `it.skip`, `it.only` | **non détecté par défaut** (Vitest les accepte) : règle ESLint `no-restricted-properties` ; ensuite refusé |
| couleur en dur (CODING UI §6) | `color: #ff0000` dans un composant | **non détecté par ESLint** (il ne lit pas le CSS d'un composant) : test Vitest `jetons.test.ts` ; ensuite refusé |
| `pnpm audit` | `lodash 4.17.20` | refusé (5 avis, dont 2 de gravité haute) |
| délai de 7 jours | `pnpm add vitest@4.1.12` (publiée 3 jours plus tôt) | refusé (`minimumReleaseAge`) ; pour une plage de versions, pnpm choisit la plus récente assez ancienne |
| verrou | `pnpm install --frozen-lockfile` après modification de `package.json` | refusé |
| scripts d'installation | dépendance avec script d'installation | refusé (`ERR_PNPM_IGNORED_BUILDS`) tant que non approuvée |
| actions épinglées | `uses: actions/…@v4` | refusé par `tools/pin-actions.py --check` ; épinglée par SHA : accepté |

### Au niveau de `make check`

Les mêmes violations, rejouées sur une copie du dépôt avec les fichiers finaux de cette étape (`deny.toml`, `clippy.toml`, `tools/`, lints du workspace) et les deux projets aux emplacements de l'ADR 0010 : **35 essais, tous conformes à l'attendu**. 32 violations font échouer `make check` (code de sortie 2) avec le nom du lint ou de la règle visé dans la sortie ; 3 cas conformes passent (état de départ, `cfg(target_os)` dans `breathly-platform`, `unsafe` justifié dans `breathly-platform`). Un essai de base dit aussi que `make check` et `make audit` sont verts sur le projet non modifié (tests, format, lints, `cargo deny`, `check-rust.py`, `pnpm audit`). Détail des violations : format, `pedantic`, `unwrap`, complexité, fonction trop longue, `unsafe` hors platform, `allow` dans un crate `forbid`, `match` avec `_`, `allow` sans raison, `Command` hors platform, test en échec, quatre dépendances interdites ou dans le mauvais sens, `cfg` selon l'OS, `#[ignore]` nu et avec raison, imbrication, trois fautes de table `[lints]` ; côté interface : format, `any`, `@ts-ignore`, complexité, `switch` incomplet, import de Tauri hors de `api/`, `it.skip`, erreur de type, test en échec, couleur en dur. `cargo audit` : essayé à part (`make audit` échoue sur `smallvec =1.6.0`).

## Ce qui a changé dans le dépôt à cause des sondes

- **Lints Rust** : `excessive_nesting`, `cognitive_complexity`, `wildcard_enum_match_arm`, `allow_attributes_without_reason` ajoutés (`Cargo.lints.fragment.toml`), seuils et interdictions dans `clippy.toml`.
- **`deny.toml`** (nouveau) : SEC1 et sens des dépendances. `CODING.md` et l'ADR 0002 disaient que le compilateur vérifie le sens des dépendances : c'est faux (il n'interdit que les cycles) ; textes corrigés.
- **`tools/check-rust.py`** (nouveau) et `tools/stacks/rust.sh` : `stack_arch` lance `cargo deny` puis ce script ; `cargo-deny` et `cargo-audit` absents = échec explicite.
- **CI** (`.github/workflows/ci.yml`) : le Rust vient de `rust-toolchain.toml` (l'ancien « stable » aurait ignoré la version fixée), `cargo-audit` et `cargo-deny` sont installés (cache), le verrou pnpm est cherché **à la racine** (l'espace de travail pnpm n'a pas de verrou par application), Node 24 comme sur le poste éprouvé, `actions/cache` épinglée par SHA.
- **`.gitignore`** : `node_modules/` à la racine (pnpm en espace de travail y met son dossier).
- **Interface** : les configurations éprouvées sont dans `outils-etape-4/interface/` ; l'étape 5 les recopie.

## Non vérifié

- **La CI hébergée.** Les étapes ajoutées à `ci.yml` n'ont pas tourné sur un exécuteur GitHub. Elles sont écrites d'après les commandes éprouvées en local ; le premier passage de la CI, ou `make ci-local`, le dira.
- **Windows.** Tout a été éprouvé sous Linux. `cargo-deny` est configuré pour la cible Windows (l'arbre de `tauri` a été résolu pour elle) mais rien n'a été compilé ni testé sous Windows.
- **Les dépendances réelles du produit** (`sysinfo`, `jwalk`, `trash`, `thiserror`, `anyhow`, `tracing`) : la liste d'interdits n'a été confrontée qu'à `tauri`. L'ajout de chacune se vérifiera à l'étape 5 (`cargo deny` dira si l'une d'elles tire une bibliothèque réseau).
- **`cargo audit` sur les avis « non maintenu »** : `cargo audit` ne fait échouer que sur une vulnérabilité ; les avertissements (crate non maintenue) s'affichent sans bloquer. `--deny warnings` n'est pas activé : à décider en voyant l'arbre réel de `tauri` à l'étape 5.
- **La documentation des outils** n'a pas été relue ligne à ligne : le comportement a été établi par essai (c'est ce qui compte pour une garde), pas par lecture. Le comportement de `wrappers` et de `excessive_nesting` ci-dessus en est un exemple : leur sens exact a été trouvé par essai.
- **Les gardes « à écrire avec le code »** (types générés à jour, clés de traduction FR/EN, test d'absence de connexion US-071) n'existent pas encore.
- **Deux gardes restent humaines**, faute d'outil sans dépendance de plus : chaîne affichée en dur dans un composant, taille d'un fichier Rust.

## Rejouer les sondes

`outils-etape-4/sondes/` contient les scripts d'essai (`probe.sh` et `mutations.sh` pour Rust, `uprobe.sh` et `umutations.sh` pour l'interface, `mkprobe.sh` et `run-make-probes.sh` pour `make check`). Ils supposent un dossier de travail avec `probe-rust/`, `probe-ui/` et `repo-copy/` ; ce sont des **scripts de référence**, pas des outils du dépôt (`make` ne les lance pas). `outils-etape-4/interface/` contient les configurations de l'interface (`package.reference.json` est le `package.json` de la sonde).

## Revue de la PR (2026-10-18)

Relecture indépendante avant fusion. Rien de bloquant (aucun invariant d'`AGENTS.md`, aucune faille, aucune exigence de la spec en défaut). Deux corrections appliquées dans la même PR :

- **Garde `cfg` selon l'OS** : l'expression régulière de `tools/check-rust.py` laissait passer `cfg!(windows)`, `cfg!(unix)` et `#[cfg_attr(windows, …)]` (seul `#[cfg(...)]` et `target_os =` étaient couverts). Corrigée (`cfg`, `cfg_attr` et la forme macro avec `!`), puis rejouée sur un workspace d'essai : les sept formes violatrices sont refusées, `cfg!(feature = …)` et `#[cfg(test)]` passent, `cfg!(windows)` est accepté dans `breathly-platform`.
- **CI, composants de Rust** : `rustup toolchain install --profile minimal` installe bien le toolchain désigné par `rust-toolchain.toml` (essai local avec rustup 1.29.0 : « overridden by … rust-toolchain.toml »), mais un profil minimal n'a ni `clippy` ni `rustfmt`. La CI les demande maintenant explicitement (`-c clippy,rustfmt`) et l'ADR 0011 impose les mêmes `components` dans `rust-toolchain.toml`. La documentation de rustup relue ce jour ne décrit pas la commande sans argument : le comportement vient de l'essai.

Vérifié à la revue : `make check` vert, `tools/pin-actions.py --check` vert (l'action `actions/cache` est épinglée), liens de documentation sans erreur, `gate` vert sur la PR (les étapes Rust de `ci.yml` n'y tournent pas, l'application n'existant pas encore : première exécution réelle à l'étape 5).
