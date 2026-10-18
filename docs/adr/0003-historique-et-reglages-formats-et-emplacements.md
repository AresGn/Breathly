# ADR 0003 : Historique et réglages : formats et emplacements

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

L'historique (S6, US-040, US-041) garde la trace de chaque action pour la preuve « rien hors règle touché » et l'annulation. Les réglages (US-002 à US-004, US-020, US-035, US-121) gardent langue, thème, mode simple ou avancé, seuils, liste d'exclusion et liste blanche. Aucune donnée ne sort de la machine (US-071). Décisions du 2026-10-18 : historique conservé 12 mois ; l'utilisateur n'efface que les entrées qui ne sont plus annulables.

## Options

Historique : **JSON Lines**, SQLite, un fichier JSON unique. Réglages : **TOML**, JSON.

## Décision

- **Historique en JSON Lines** : un fichier texte, une entrée JSON par ligne, ajoutée en fin de fichier. Champs : `v` (version du schéma), `ts` (UTC, RFC 3339), `action` (`clean`, `restore`, `purge_trash`), `rule_id`, `path`, `size_bytes`, `destination` (`trash` ou `permanent`), `trash_ref` (pour restaurer), `outcome` (`pending`, `ok`, `skipped`, `error`, `uncertain`). Une entrée `pending` est écrite avant l'action, puis son résultat est inscrit ; au démarrage suivant un arrêt brutal, une entrée restée `pending` est vérifiée (design 0001, §3). Lisible dans un éditeur, sans dépendance de base de données.
- **Rotation et effacement** : réécriture complète vers un fichier temporaire puis renommage, pour qu'un arrêt brutal ne tronque jamais le fichier. Suppression des entrées de plus de 12 mois (valeur de départ) et, à la demande, des entrées non annulables (US-040.3, US-040.4).
- **Réglages en TOML**, même langage que les règles. Fichier lu au démarrage, écrit par écriture atomique.
- **Emplacements** : dossiers utilisateur de l'OS, jamais le dossier d'installation. Linux : `$XDG_CONFIG_HOME/breathly/` (réglages) et `$XDG_STATE_HOME/breathly/` (historique) ; Windows : `%APPDATA%\Breathly\` (réglages) et `%LOCALAPPDATA%\Breathly\` (historique). **Non vérifié** à ce jour : le crate qui fournira ces chemins, et la valeur par défaut de `XDG_STATE_HOME` ; à confirmer à l'implémentation (prototype 2.9).

## Conséquences

- Un seul écrivain à la fois sur l'historique (verrou de fichier) ; l'interface et le service Breathly passent par `breathly-disk`, qui est le seul à écrire.
- Un schéma versionné (`v`) permet de lire d'anciennes entrées après une mise à jour.
- Si le volume d'entrées devient un problème, passer à SQLite sera un nouvel ADR ; le format JSON Lines peut s'importer.
- Les chemins sont stockés en clair mais restent sur la machine (US-040.2, US-071).
