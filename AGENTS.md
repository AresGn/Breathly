# AGENTS.md : invariants du projet Breathly

Court par conception. Les détails vivent dans `docs/`. Ce fichier est lu à chaque session d'agent et à chaque arrivée d'une personne sur le projet. **Une règle écrite ici doit avoir un garant** (une vérification automatique ou une ligne de relecture) : sinon elle n'a pas sa place ici.

Références : `GLOSSARY.md` (vocabulaire), `docs/PRD.md` (exigences, brouillon v0) et `docs/prd/` (gabarit des PRD par fonctionnalité), `docs/adr/` (décisions), `docs/README.md` (où est quoi), `CONTRIBUTING.md` (règles de travail).

## Stack

- Cœur : Rust (édition 2024), workspace Cargo (`crates/`), PRD §10
- Interface : Tauri v2, interface web légère (Svelte 5, ADR 0001), `apps/desktop/`
- Extension navigateur : TypeScript, Manifest V3, une base de code pour Firefox et Chromium, `extension/`
- Règles de nettoyage et définitions de navigateurs : fichiers TOML (`rules/`, `browsers/`), modifiables sans écrire de Rust
- Aucun serveur, aucun compte, aucune télémétrie (PRD §0 point 4)

Applications déclarées dans `.kit.conf` (`APPS`) ; chacune a son `CODING.md`.

## Architecture

Workspace Cargo : un crate par module du PRD §5 (`breathly-platform`, `-rules`, `-diagnose`, `-disk`, `-guard`, `-daemon`, `-nmhost` ; V0.2 à V0.4 : `-watch`, `-startup`, `-tune`, `-elevated`). Le sens des dépendances est celui du tableau du PRD §5 ; il ne s'inverse jamais.

- `platform` et `rules` ne dépendent d'aucun autre crate du projet.
- `app-ui` appelle les modules uniquement par des commandes Tauri : aucune règle métier dans l'interface web.
- Les fonctions qui diffèrent selon l'OS vivent derrière un trait dans `platform` ; le reste du code ne contient pas de `cfg(target_os)`.
- Un module ne touche pas aux éléments internes d'un autre (`pub(crate)` par défaut).

## Invariants métier (non négociables)

Breathly agit sur les fichiers de personnes non techniques : une violation est un incident, pas un bug.

1. Rien n'est supprimé sans afficher la liste et obtenir une confirmation (PRD §0 point 6, §4).
2. Les fichiers de l'utilisateur vont dans la corbeille du système ; seuls les caches 🟢 et le vidage confirmé de la corbeille se suppriment définitivement, et l'écran le dit (S3).
3. Garde-fous non désactivables (S5) : jamais hors des emplacements prévus par une règle ; jamais les profils de navigateur, `Documents`, `Images`, `Vidéos`, `Musique`, `Bureau`, les conteneurs chiffrés, les dossiers `.git`, les fichiers ouverts.
4. Trois niveaux de risque partout (🟢 🟡 🔴) ; un élément 🔴 n'est jamais sélectionnable.
5. Chaque action est inscrite à l'historique (S6).
6. Les URL ne sortent jamais du navigateur : l'extension ne transmet que le domaine et des compteurs (T3).
7. Aucune donnée ne quitte la machine : pas de télémétrie, pas de requête réseau non demandée.
8. Une action qui exige les droits administrateur passe par l'élévation standard de l'OS, une action à la fois, après une explication ; jamais l'application entière en administrateur ; Breathly ne manipule jamais un mot de passe ; en repli, mode guidé (commande à copier, rien d'exécuté) (PRD §7.8).
9. Un navigateur se ferme proprement, jamais en tuant le processus (B3).
10. Breathly n'est pas un antivirus : il ne dit jamais « virus » ni « sain » ; un processus inconnu est « à examiner » (PRD D6). Il ne ferme jamais un logiciel sans clic de l'utilisateur (PRD §6 V0.2).
11. Aucune règle de nettoyage ou de réglage système n'est ajoutée sans explication en français et en anglais, niveau de risque et chemins par OS (validé par un test).

## Ports à implémenter (adapters remplaçables)

Seules les dépendances à l'OS et à l'horloge passent par un trait avec un adapter remplaçable (et un faux pour les tests) : `Clock`, `TrashProvider`, `ProcessInspector`, `DiskInspector`, `StartupManager`. Les tests ne touchent jamais au vrai disque de l'utilisateur : ils travaillent dans un dossier temporaire.

## Base de données

Sans objet : Breathly n'a pas de base de données serveur. L'historique des actions et les réglages sont des fichiers locaux (format à fixer par un ADR). Les règles TOML sont validées par un schéma.

## Clean code : règles communes à toutes les piles

Les seuils sont contrôlés par la machine (voir la configuration de chaque pile). La relecture couvre le reste. Le détail et les exemples sont dans le `CODING.md` de chaque application.

| Règle | Ce que ça veut dire |
|---|---|
| Pas de type flou | `any`, `mixed`, `dynamic`, `interface{}` interdits hors frontière ; toute entrée est validée et typée tout de suite |
| Types distincts | identifiants et valeurs métier en types dédiés (objet-valeur, type brandé), jamais une chaîne partout |
| États en énumération | `enum` / union fermée et `match` / `switch` **exhaustif**, sans cas par défaut |
| Échec métier attendu | valeur de retour typée (résultat, enum de raison), pas exception ; exception seulement pour l'invariant violé |
| Jamais d'erreur avalée | pas de capture vide ; contexte ajouté puis relance, ou décision explicite écrite |
| Taille | fonction ≤ 40 lignes, complexité ≤ 10, imbrication ≤ 3 (retours anticipés), fichier ≤ 250 à 300 lignes |
| Paramètres | ≤ 3 positionnels ; au-delà, un objet de commande ou des paramètres nommés |
| Booléens | nommés `is`, `has`, `can`, `should` ; jamais muets en paramètre |
| Immutabilité | immuable par défaut ; une transition retourne un nouvel état |
| Dépendances inversées | port (interface) dans le domaine, adapter dans l'infrastructure, branchement dans la composition |
| Pas de verbosité | pas de commentaire qui répète le code, d'abstraction « au cas où », de couche de délégation sans valeur |

## Sécurité par conception

Règles techniques SEC1 à SEC11 (aucun réseau, suppression par règle et garde-fous, l'interface n'envoie jamais un chemin, historique avant action, fichiers locaux non fiables, élévation, communication locale, dépendances, distribution) : `docs/policies/securite.md`. Une règle de ce fichier ou de ce document ne se contourne pas pour passer au vert.

## Écrire du code dans une application

Avant de coder dans une application, lire son `CODING.md` (chemin déclaré dans `.kit.conf`). Il dit où va chaque chose, comment écrire une fonction, les erreurs, les tests et la définition de « terminé ».

## Commandes

```
make hooks    # une fois par clone
make check    # secrets + lint + statique + architecture + tests de chaque application
make ci-local # rejoue tout le job gate puis publie le statut local-ci (porte de fusion quand la CI hébergée est indisponible)
make help     # toutes les commandes
```

Avant de passer une PR « prête » : `make ci-local` vert (arbre propre, commit poussé). Ne jamais publier le statut `local-ci` à la main ni sans avoir lancé le script.

## Revue de code

Chaque constat est vérifié dans le document source qu'il cite (PRD, ADR, design) avant publication. Bloquant : invariant de ce fichier, sécurité, exigence de la spec qui concerne cette PR ; le reste est une suggestion (détail : `CONTRIBUTING.md` §10). Aucune signature ni mention d'outil dans les commits, PR et commentaires.

## Agent skills

### Issue tracker

Les tickets sont des issues GitHub ; un ticket par application (`app:<nom>`), chacun bloqué par le ticket de contrat de sa fonctionnalité. Voir `docs/agents/issue-tracker.md`.

### Triage labels

Les cinq étiquettes par défaut (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`) plus `en-attente-decision`, sans mention d'outil dans les commentaires. Voir `docs/agents/triage-labels.md`.

### Domain docs

Un seul contexte : `GLOSSARY.md` et `docs/adr/` à la racine. Voir `docs/agents/domain.md`.

## Avant toute PR

Remplir la checklist de `.github/pull_request_template.md`. Une violation d'un invariant, une suppression hors d'une règle, un garde-fou contourné ou un test qui touche au vrai disque sont des défauts bloquants.
