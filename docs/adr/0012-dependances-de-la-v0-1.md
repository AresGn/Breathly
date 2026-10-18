# ADR 0012 : Dépendances de la V0.1

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le PRD §14 demande l'accord du propriétaire avant tout ajout de dépendance, et l'ADR 0011 n'a accepté que `thiserror`, `anyhow`, `tracing` (et `tempfile` en développement). Le PRD §10 prévoit `sysinfo`, `jwalk` et `trash`. Il reste des besoins qu'aucun document n'a tranchés : lire et écrire du TOML et du JSON, résoudre les dossiers standard, ouvrir un dossier dans l'explorateur, écrire un fichier de façon atomique, horodater, identifier les entrées d'historique, brancher un abonné aux journaux, et deux plugins de l'interface. Ticket : #24.

**Méthode.** Versions, dates de publication et licences relevées sur crates.io le 2026-10-18 (API publique). Le délai du kit (`MIN_RELEASE_AGE_DAYS = 7`) vaut pour les versions retenues : Cargo ne l'applique pas, les dates ont été comparées à la main. Nombre de crates transitives mesuré par `cargo tree` sur un projet d'essai (Linux et `x86_64-pc-windows-msvc`). `cargo deny check bans sources`, avec le `deny.toml` du dépôt, passe sur le projet d'essai qui réunit toutes les candidates : aucune bibliothèque réseau (SEC1) n'entre.

**Constats utiles.**
- `serde` 1.0.229, `serde_json` 1.0.152, `toml` 1.1.8, `dirs` 7.0.0, `time` 0.3.55 sont **déjà dans `Cargo.lock`** (tirés par Tauri) : leur coût marginal est nul. `option-ext` (MPL-2.0, copyleft de fichier) y est déjà aussi, par `dirs`.
- `sysinfo` 0.39.6 et `trash` 5.2.9 ont servi aux prototypes P1 et P2 (`docs/research/2026-10-18-prototypes-2-9.md`). `jwalk` 0.9.0 n'a pas été essayé.
- `std::fs::File::try_lock` compile et s'exécute avec Rust 1.97.1 sous Linux (essai du 2026-10-18).
- `dirs` 7.0.0 fournit `document_dir`, `picture_dir`, `video_dir`, `audio_dir`, `desktop_dir`, `data_local_dir` (code source lu) ; `state_dir` renvoie `None` sous Windows.
- `open` 5.4.4 essaie `xdg-open`, `gio`, `gnome-open`, `kde-open` sous Linux (code source lu).
- Les plugins `tauri-plugin-*` publiés le 2026-10-14 (opener 2.7.2, clipboard-manager 2.5.0, dialog 2.8.3) n'ont que 4 jours : seules les versions antérieures d'au moins 7 jours sont retenables.

**Non vérifié.** Comportement de `open` sous Wayland, snap, Flatpak et sous Windows ; `File::lock` et `rename` sur un fichier existant sous Windows ; `navigator.clipboard` sous WebKitGTK et sous WebView2 ; `jwalk` sur 500 000 fichiers (mesure du ticket #48) ; les dépôts de `dirs` ont migré vers Codeberg (relevé crates.io), sans conséquence vérifiée sur la maintenance.

## Options

| Besoin | Options examinées | Retenue |
|---|---|---|
| Sérialisation | `serde` + `serde_json` + `toml` ; format maison | serde, serde_json, toml |
| Système, disque, corbeille | `sysinfo`, `jwalk`, `trash` ; tout maison (`/proc`, API Windows en `unsafe`, corbeille à la main) ; parcours avec la bibliothèque standard sans `jwalk` | les trois |
| Dossiers standard | `dirs` ; `etcetera` + code maison ; variables d'environnement seules | `dirs` |
| Ouverture d'un dossier | `open` ; `std::process::Command` maison ; `tauri-plugin-opener` (tire `opener`, 45 crates dont `url` et ICU) | `open` |
| Fichiers de l'historique et des réglages | bibliothèque standard + `time` + compteur ; `tempfile` + `uuid` + `time` ; `atomic-write-file` + `chrono` + `ulid` | standard + `time` + compteur |
| Abonné aux journaux | `tracing-subscriber` ; abonné maison ; aucun abonné | `tracing-subscriber` |
| Plugins Tauri | dialogue ; presse-papiers ; aucun ; les deux | dialogue seulement |

## Décision

Sont acceptées pour la V0.1, en plus de l'ADR 0011 :

| Dépendance | Version minimale | Publiée le | Licence | Crate du projet |
|---|---|---|---|---|
| `serde` (fonction `derive`) | 1.0.229 | 2026-07-18 | MIT OR Apache-2.0 | rules, disk, desktop |
| `serde_json` | 1.0.152 | 2026-10-10 | MIT OR Apache-2.0 | disk, desktop |
| `toml` | 1.1.8 | 2026-10-09 | MIT OR Apache-2.0 | rules, desktop |
| `sysinfo` | 0.39.6 | 2026-07-09 | MIT | platform |
| `trash` | 5.2.9 | 2026-09-13 | MIT | platform |
| `jwalk` | 0.9.0 | 2026-08-05 | MIT | disk (sous réserve de la mesure du ticket #48) |
| `dirs` | 7.0.0 | 2026-09-05 | MIT OR Apache-2.0 | platform |
| `open` | 5.4.4 | 2026-09-10 | MIT | platform |
| `time` (fonctions `formatting` et `parsing`) | 0.3.55 | 2026-08-01 | MIT OR Apache-2.0 | platform, disk |
| `tracing-subscriber` (sans fonctions par défaut, `fmt` seule) | 0.3.23 | 2026-03-13 | MIT | desktop |
| `tauri-plugin-dialog` | 2.8.1 | 2026-10-01 | Apache-2.0 OR MIT | desktop |

Pas de dépendance pour : l'écriture atomique (fichier temporaire voisin, `sync_all`, `rename` de la bibliothèque standard), le verrou de fichier (`File::lock` de la bibliothèque standard), l'identifiant d'une entrée d'historique (compteur croissant lu dans le fichier, déterministe avec le faux `Clock`), le presse-papiers (`navigator.clipboard` à essayer d'abord).

La répartition par crate est celle prévue ; une PR peut la déplacer sans nouvel accord tant qu'elle n'ajoute aucune dépendance hors de cette liste et respecte le sens des dépendances de `deny.toml`. Toute autre dépendance se redemande (PRD §14).

## Conséquences

- Les tickets d'implémentation ajoutent ces dépendances dans leur PR, avec `--locked` et `cargo deny` verts. Le verrou ne se fusionne qu'après vérification de la date de publication de chaque version nouvelle (Cargo n'a pas de délai de 7 jours).
- `sysinfo` ajoute 3 crates sous Linux et 24 sous Windows, `trash` 9 et 22, `jwalk` 12 (avec `rayon`), `open` 5, `tracing-subscriber` minimal 7 (mesures indicatives sur le projet d'essai).
- `option-ext` (MPL-2.0) reste dans l'arbre par `dirs`, comme aujourd'hui par Tauri. Le projet est sous licence MIT ; le copyleft de la MPL-2.0 ne s'applique qu'aux fichiers de ce crate. Aucun contrôle de licences n'est en place dans `deny.toml` : à décider avant la première release (SEC11).
- Le **presse-papiers** : si `navigator.clipboard` échoue sous l'un des deux moteurs, `tauri-plugin-clipboard-manager` (2.4.1, publié le 2026-10-01 au moment de cet ADR) s'ajoute par une note datée sur cet ADR.
- Ouvrir un dossier lance un programme externe : seul `breathly-platform` le fait (`CODING.md` §6), avec un `#[allow(…, reason)]` relu.
- La mesure du ticket #48 peut retirer `jwalk` : le parcours repasse alors sur la bibliothèque standard, sans nouvel ADR.
- L'historique garde le verrou de fichier de l'ADR 0003 ; le choix d'un compteur pour l'identifiant des entrées est repris par l'addendum à l'ADR 0003 du ticket d'historique.

## Impact sur l'architecture

Aucun changement du graphe de dépendances entre crates (design 0002 §3). Les dépendances système restent derrière les traits de `breathly-platform`, qui est le seul crate à pouvoir contenir du `unsafe` et à lancer un programme. Cette décision débloque le ticket #24 et, avec lui, les tickets d'implémentation qui l'attendaient (schéma des règles, chemins standard, adapters, réglages, historique) : `make blockers` donne la liste après sa fermeture.
