# ADR 0004 : Chargement des règles de nettoyage

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

G6 demande que des contributeurs ajoutent des règles de nettoyage sans écrire de Rust. S2 interdit toute suppression hors règle explicite. Question de sécurité : si les règles étaient lues depuis un dossier modifiable, un logiciel malveillant qui y écrit pourrait faire supprimer des fichiers (les garde-fous S5 limiteraient le dégât).

## Options

1. **Règles intégrées au binaire à la compilation**, validées par la CI.
2. Dossier de règles lu à l'exécution.
3. Les deux.

## Décision

Option 1 pour la V0.1 (US-060.1). Les fichiers `rules/*.toml` sont validés par la CI (schéma, chemins, explications FR et EN, niveau de risque) puis intégrés au binaire à la compilation ; mécanisme d'intégration (`build.rs` ou équivalent) choisi à l'implémentation. Aucun dossier de règles modifiable n'est lu à l'exécution. Le champ `command` est lu et affiché en mode avancé, jamais exécuté (US-060.4).

## Conséquences

- Un contributeur ajoute un fichier TOML par PR, sans toucher à Rust ; l'utilisateur reçoit la règle avec la version suivante.
- La désérialisation refuse tout champ inconnu et toute règle incomplète : un test charge chaque règle du dépôt.
- La liste des actions administrateur (E5, ADR 0006) est un sous-ensemble de ces mêmes règles, embarqué aussi dans l'assistant d'élévation.
- Règles personnalisées de l'utilisateur : hors V0.1. Un ADR futur devra répondre à qui peut écrire dans ce dossier et comment on authentifie une règle (signature, ou limitation à des chemins sous le dossier de l'utilisateur).

## Note du 2026-10-30 : mécanisme d'intégration (ticket #32)

Le mécanisme retenu est un **`build.rs`** dans `breathly-rules` : à la compilation, il liste les fichiers `rules/*.toml` (triés par nom) et génère un tableau de `include_str!`. `load_cleanup_rules()` valide chaque fichier avec `parse_cleanup_rules` puis l'unicité des identifiants entre fichiers. Un nouveau fichier est pris en compte sans toucher au Rust (`cargo:rerun-if-changed` sur le dossier) ; une entrée qui ressemble à une règle sans en être une (dossier `x.toml`, extension `.TOML`) arrête la compilation. Garde-fous : le test `every_rule_of_the_repository_is_valid` charge toutes les règles (un fichier invalide fait échouer `make check` et la CI, message qui nomme le fichier et la règle) ; `every_toml_file_of_the_rules_folder_is_embedded` vérifie qu'aucun fichier n'est laissé de côté ; `the_crate_reads_no_file_at_run_time` refuse toute lecture de fichier dans le code du crate (sous-dossiers compris, hors `build.rs`). Limite : le crate ne se construit que dans le workspace (`../../rules`), ce qui suffit tant qu'il n'est pas publié (`publish = false`). Guide du contributeur : [`rules/README.md`](../../rules/README.md).
