# ADR 0005 : Vidage de la corbeille et espace libéré

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le principe 2 du PRD dit « réversible par défaut » : les fichiers de l'utilisateur vont à la corbeille. Mais un fichier mis à la corbeille ne libère aucun espace tant qu'elle n'est pas vidée, et le §8 prévoit une règle « corbeille ». S3 est réécrit le 2026-10-18 : les seules suppressions définitives sont les caches 🟢 et le vidage confirmé de la corbeille.

## Options

1. **Deux chiffres et vidage confirmé** (retenu) : l'écran distingue « envoyé à la corbeille » et « libéré maintenant » ; vider la corbeille est un élément 🟡 distinct.
2. Pas de vidage par Breathly.
3. Suppression définitive des éléments 🟡.

## Décision

Option 1. Le crate `trash` (version 5.2.9 lue sur docs.rs le 2026-10-18) fournit `trash::delete` et `delete_all`, et, **sur Windows et sur Linux (corbeille Freedesktop) uniquement**, le module `os_limited` avec `list`, `metadata`, `restore_all`, `purge_all` et `is_empty`. Breathly s'appuie sur eux pour l'annulation (US-041) et le vidage (US-037). macOS n'est pas pris en charge par `os_limited`, ce qui s'accorde avec macOS après la V1.0.

L'espace « libéré maintenant » est la somme des tailles des éléments réellement détruits (caches 🟢 supprimés, éléments purgés de la corbeille), pas la différence d'espace libre du disque, que d'autres activités faussent.

## Conséquences

- Vider la corbeille détruit des fichiers de l'utilisateur : confirmation obligatoire, nombre d'éléments et taille affichés, jamais coché par défaut, inscrit dans l'historique comme suppression définitive (US-037).
- Purger concerne la corbeille du système, qui peut contenir des éléments que Breathly n'a pas mis lui-même : l'écran le dit et ne purge que les éléments listés et confirmés. **Vérifié par le prototype P1 (Linux, 2026-10-18, voir `docs/research/2026-10-18-prototypes-2-9.md`)** : `purge_all` détruit seulement les éléments donnés, même avec un élément étranger dans la corbeille ; `restore_all` sur un emplacement occupé renvoie `RestoreCollision` sans rien écraser ; l'identifiant d'un élément est le chemin de son fichier `.trashinfo`. **Windows : non testé.**
- L'annulation échoue sans rien écraser si l'emplacement d'origine est occupé (US-041.2).
- Le critère de réussite de la V0.1 compte l'espace réellement libéré, corbeille vidée comprise (PRD §15).
