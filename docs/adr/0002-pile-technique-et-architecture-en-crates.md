# ADR 0002 : Pile technique et architecture en crates

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le PRD (§0 point 2, §5, §10) fixe la pile : cœur en Rust (édition 2024), interface Tauri v2 (ADR 0001), extension WebExtension en TypeScript. Il faut dire comment le code Rust se découpe pour que les modules du PRD ne dépendent pas les uns des autres n'importe comment, et pour tenir G5 (rester léger).

## Options

1. **Workspace Cargo, un crate par module** : frontières de dépendances vérifiables par `cargo`, compilation incrémentale, binaires séparés (service, hôte, interface).
2. **Un seul crate avec des modules** : plus simple au départ, mais rien n'empêche un module d'en importer un autre dans le mauvais sens.
3. **Plusieurs dépôts** : isole tout, mais coûte cher en coordination pour une petite équipe.

## Décision

Option 1. Un workspace Cargo à la racine : `crates/breathly-platform`, `-rules`, `-diagnose`, `-disk`, `-guard`, `-daemon` (binaire du service Breathly), `-nmhost` (binaire de l'hôte), plus `apps/desktop/` (Tauri) et `extension/`. Le sens des dépendances est celui du PRD §5 et ne s'inverse jamais : `platform` et `rules` ne dépendent d'aucun crate du projet ; `app-ui` appelle les modules par des commandes Tauri uniquement.

## Conséquences

- Les dépendances interdites sont des erreurs de compilation : un crate ne déclare dans son `Cargo.toml` que ce que le tableau du PRD §5 autorise. La CI le contrôle (étape 4 du parcours : outil à choisir et à prouver par sonde négative).
- Tout ce qui diffère selon l'OS vit derrière un trait dans `breathly-platform` (`ProcessInspector`, `DiskInspector`, `TrashProvider`, `StartupManager`, `Clock`), avec un faux pour les tests ; aucun `cfg(target_os)` ailleurs.
- Pour Tauri v2, les capacités (`capabilities`) limitent ce que l'interface web peut appeler ; la documentation indique que **par défaut toute commande enregistrée est appelable depuis toute fenêtre** : chaque commande est donc enregistrée explicitement et restreinte (source : https://v2.tauri.app/security/capabilities/, lue le 2026-10-18).
- Les crates `sysinfo`, `jwalk` et `trash` sont les dépendances prévues (PRD §10) ; l'ajout de toute autre dépendance se demande d'abord (PRD §14).

## Impact sur l'architecture

Définit la structure du dépôt (PRD §11). Un changement de découpage plus tard se fait crate par crate, sans toucher au contrat entre modules tant que les traits de `platform` restent stables.
