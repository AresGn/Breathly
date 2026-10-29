# Design 0004 : format des règles de nettoyage

- **Statut** : proposé, à approuver par le propriétaire avant fusion (PRD §14 : « changement du format des règles »)
- **Date** : 2026-10-29
- **Sources** : `docs/PRD.md` §8 et S5, `docs/stories.md` US-060.1, US-060.2, US-060.4, US-024.2, ADR 0004, ADR 0005, `docs/policies/securite.md` SEC5 et SEC6, `docs/policies/modele-de-menaces.md` TS-09, ticket #31
- **Code** : `crates/breathly-rules` (`parse_cleanup_rules`) ; tests dans `crates/breathly-rules/tests/`
- **Vocabulaire** : `GLOSSARY.md` fait foi (règle de nettoyage, niveau de risque, fichier marqueur, dossier de projet recréable).

Ce document remplace l'exemple « provisoire » du PRD §8. Il dit ce qu'un fichier `rules/*.toml` peut contenir et ce que la validation refuse. L'intégration des fichiers au binaire est le ticket #32.

## 1. Une règle

Un fichier contient une ou plusieurs tables `[[rule]]` et rien d'autre.

```toml
[[rule]]
id          = "npm-cache"
name.fr     = "Cache de npm"
name.en     = "npm cache"
explain.fr  = "Paquets téléchargés par npm. Ils sont retéléchargés automatiquement si besoin."
explain.en  = "Packages downloaded by npm. They are re-downloaded automatically when needed."
risk        = "safe"                                   # safe | review | personal
os          = ["linux", "windows"]
kind        = "folder"                                 # folder | marked-folder | files-by-extension | system-trash
paths.linux   = ["~/.npm/_cacache"]
paths.windows = ["%LOCALAPPDATA%/npm-cache/_cacache"]
command     = "npm cache clean --force"                # facultatif, affiché en mode avancé, jamais exécuté
```

| Champ | Obligatoire | Contenu |
|---|---|---|
| `id` | oui | minuscules ASCII, chiffres, tirets isolés ; 64 caractères au plus ; unique dans l'ensemble des règles |
| `name.fr`, `name.en` | oui | nom court, non vide |
| `explain.fr`, `explain.en` | oui | explication en langage simple, non vide (US-024.2) |
| `risk` | oui | `safe` 🟢, `review` 🟡, `personal` 🔴 (jamais sélectionnable) |
| `os` | oui | `linux`, `windows` ; au moins un, sans doublon (macOS : après la V1.0, PRD §0) |
| `kind` | oui | une des quatre formes du §2 ; jamais déduite |
| `paths.linux`, `paths.windows` | selon la forme | une liste non vide **par système de `os`**, et aucune pour un système absent de `os` |
| `folder`, `marker` | forme `marked-folder` | un nom simple chacun |
| `extensions` | forme `files-by-extension` | liste non vide, sans doublon |
| `command` | non | texte non vide ; **lu et conservé pour l'affichage, jamais exécuté** (US-060.4, SEC6, TS-09) |

Rien n'est corrigé en silence (SEC5) : un champ inconnu, un champ d'une autre forme, un doublon ou une valeur vide refusent la règle.

## 2. Les quatre formes

| `kind` | Ce qui est proposé | Champs propres | Exemple (PRD §8) |
|---|---|---|---|
| `folder` | le contenu d'un dossier entier | `paths` | caches npm, pip, navigateurs |
| `marked-folder` | un dossier nommé `folder` sous l'un des `paths`, seulement si le fichier `marker` est à côté de lui | `folder`, `marker`, `paths` (dossiers de recherche) | `node_modules` + `package.json`, `target` + `Cargo.toml` (🟡) |
| `files-by-extension` | les fichiers d'un des `paths` dont l'extension est listée | `extensions`, `paths` | installeurs `.deb`, `.exe`, `.msi`, `.appimage` de Téléchargements (🟡) |
| `system-trash` | le contenu de la corbeille du système (vidage confirmé, ADR 0005) | **aucun** : un chemin, même vide, refuse la règle | corbeille (🟡) |

- `folder` et `marker` sont un seul nom : pas de `/`, pas de `.` ni `..`, pas de `.git`, pas d'un dossier protégé.
- `extensions` : minuscules ASCII et chiffres, sans le point, 16 caractères au plus. La comparaison avec les fichiers du disque se fera sans tenir compte de la casse (`.AppImage`). Les extensions de conteneurs chiffrés (`hc`, `vc`, `kdbx`) sont refusées (S5).

## 3. Chemins autorisés

Un chemin s'écrit avec `/` comme séparateur, sur tous les systèmes, et commence par l'une de ces racines :

| Racine | Système | Justification |
|---|---|---|
| `~` | Linux | les caches de l'utilisateur sont sous son dossier personnel (`~/.cache`, `~/.npm`, `~/.gradle`, `~/.cargo/registry`) ; Breathly ne tourne jamais en administrateur (invariant 8) |
| `%LOCALAPPDATA%` | Windows | caches des outils et des navigateurs, `CrashDumps`, miniatures |
| `%APPDATA%` | Windows | caches de quelques outils (dont npm selon la version) et de VS Code |
| `%TEMP%` | Windows | fichiers temporaires (PRD §8) |
| `%USERPROFILE%` | Windows | outils qui écrivent directement dans le dossier personnel (`.gradle`, `.cargo`), Téléchargements |

Sont refusés (`PathRefusal`) :

- tout autre début : chemin absolu, racine du disque (`/`, `C:/`), variable non listée (`$HOME`, `%PROGRAMDATA%`), chemin relatif ;
- une racine de l'autre système (`%TEMP%` dans `paths.linux`) ;
- **la racine seule** (`~`, `%USERPROFILE%`) pour une cible nettoyée ; elle n'est permise que comme dossier de recherche d'une règle `marked-folder`, où seul le dossier nommé, reconnu par son fichier marqueur, est proposé ;
- un nom vide, `.` ou `..` (pas de remontée, pas de `//`, pas de `/` final) ;
- `\`, `:`, `*`, `?`, `<`, `>`, `"`, `|` et les caractères de contrôle : pas de motif, pas de lecteur, pas de flux NTFS ;
- un premier dossier protégé sous `~` ou `%USERPROFILE%` : `Documents`, `Images`, `Pictures`, `Vidéos`, `Videos`, `Musique`, `Music`, `Bureau`, `Desktop`, sans tenir compte de la casse (S5) ;
- un dossier `.git` à n'importe quel niveau (S5).

`Windows.old` (V0.4, droits administrateur) n'a pas de racine autorisée en V0.1 : il en faudra une, avec l'ADR 0006.

**Ce que la validation statique ne garantit pas**, et qui reste aux garde-fous de `breathly-disk`, revérifiés pour chaque élément au moment d'agir (S5, SEC2) : les profils de navigateur (sous Windows, le cache de Chromium est **dans** le dossier du profil, `User Data/Default/Cache` : un refus par préfixe bloquerait des règles légitimes), les conteneurs chiffrés trouvés dans un dossier, les fichiers ouverts, les liens symboliques et les jonctions (ADR 0019), les dossiers de l'utilisateur déplacés ou renommés par la langue du système.

## 4. Erreurs

`parse_cleanup_rules` s'arrête à la première erreur. Une règle invalide est nommée par son `id` tel qu'écrit, ou par sa position (« n° 3 ») quand il manque (US-060.2). Chaque erreur a un code stable `rules.<cas>` et une clé de traduction `errors.rules.<cas>` (CODING.md §5).

| Code | Cas |
|---|---|
| `rules.unreadable` | le fichier n'est pas du TOML |
| `rules.unknown_top_level_key`, `rules.no_rule` | clé hors d'une règle, aucune règle |
| `rules.duplicate_id` | deux règles avec le même `id` |
| `rules.unknown_field` | champ inconnu, y compris dans `name`, `explain`, `paths` |
| `rules.missing_field`, `rules.missing_text` | champ obligatoire absent ; texte FR ou EN absent ou vide |
| `rules.wrong_type`, `rules.invalid_id`, `rules.empty_value`, `rules.duplicate_value` | valeur mal formée |
| `rules.unknown_risk_level`, `rules.unknown_os`, `rules.unknown_kind` | valeur hors de la liste |
| `rules.field_not_allowed_for_kind`, `rules.paths_for_system_trash` | champ d'une autre forme |
| `rules.missing_paths_for_os`, `rules.paths_for_unlisted_os`, `rules.invalid_path` | chemins |
| `rules.invalid_name`, `rules.invalid_extension` | `folder`, `marker`, `extensions` |

Ces messages s'adressent aux contributeurs (validation en CI, ADR 0004) ; l'utilisateur ne voit jamais une règle invalide, puisqu'elle n'entre pas dans le binaire.

## 5. Questions ouvertes

- Faut-il interdire `Downloads`/`Téléchargements` comme cible d'une règle `folder` (dossier entier), pour ne l'autoriser qu'aux règles `files-by-extension` ? Non fait : S5 ne le liste pas.
- Faut-il ajouter `OneDrive` aux dossiers protégés sous `%USERPROFILE%` ? Non fait pour la même raison.
