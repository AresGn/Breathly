# Coquille de l'application de bureau : ce qu'elle coûte et ce qu'elle a révélé

- **Date** : 2026-10-18
- **Question** : que coûte une fenêtre Tauri v2 + Svelte 5 minimale (P4 du parcours, étape 2.9), et la chaîne d'outils fonctionne-t-elle de bout en bout (construire, contrôler, auditer) ?
- **Poste** : Linux Mint, X11, 16 Go de RAM, WebKitGTK 2.50.6 (`libwebkit2gtk-4.1-dev`), Rust 1.97.1, Node 24.16.0, pnpm 11.5.3.
- **Objet mesuré** : la coquille de la PR « coquille desktop » : une fenêtre 960 × 640 qui affiche « Breathly » avec les jetons de design, en version `release`, avec la politique de contenu (CSP) du dépôt.
- **Alimente** : ADR 0008 (budgets), `docs/policies/securite.md` (SEC1, SEC10, SEC11).

## Mesures

| Mesure | Valeur | Cible du PRD (§9) |
|---|---|---|
| Fenêtre visible après le lancement | 0,28 s | — |
| Taille de la fenêtre à l'ouverture | 960 × 640 (conforme au design 0003) | — |
| Processus de la fenêtre | 3 : `breathly-desktop`, `WebKitNetworkProcess`, `WebKitWebProcess` | — |
| Mémoire proportionnelle (PSS), total des 3 processus | **288 Mo** (96 + 36 + 156) | interface ouverte < 150 Mo : **dépassée** |
| Mémoire résidente (RSS), total | 461 Mo (166 + 71 + 224) | idem : **dépassée** |
| Processus restants après la fermeture | aucun | aucun processus fenêtre fermée : **tenu** (US-072.1) |
| Binaire `release` (Rust, strippé) | 10,7 Mo | — |
| Installeur `.deb` | 2,78 Mio | < 15 Mo : tenu |
| Installeur `.rpm` | 2,78 Mio | < 15 Mo : tenu |
| `.AppImage` | **98,4 Mio** | < 15 Mo : **dépassée** (l'image embarque WebKitGTK et ses bibliothèques) |

Méthode : le programme lance le binaire, attend l'apparition de la fenêtre (`xdotool`), laisse 6 s, puis lit `/proc/<pid>/smaps_rollup` pour chaque processus de l'arbre. La PSS répartit la mémoire partagée entre les processus qui la partagent ; la RSS la compte entièrement chaque fois. Le texte de la fenêtre est celui du fichier d'essai : une page d'un titre, sans écran. Une capture de la fenêtre confirme que la page s'affiche correctement sous la CSP : titre centré, couleurs du thème clair.

**Lecture.** Même vide, une fenêtre Tauri sous Linux dépasse de loin la cible de 150 Mo du PRD, parce que la page web tourne dans WebKitGTK (deux processus supplémentaires, 190 Mo de PSS à eux deux). Ce n'est pas un défaut de la coquille, c'est le coût du moteur web du système. L'ADR 0008 doit en tirer une décision : réviser la cible, ou reconsidérer le choix d'une interface web (ADR 0001). **Windows (WebView2) n'est pas mesuré.**

## Comparaison : fenêtre native minimale (P6)

Pour que la décision sur G5 repose sur un chiffre comparable, une fenêtre `egui` (`eframe` 0.37, réglages par défaut, version `release`, 960 × 640 logiques, un titre centré) a été mesurée sur la même machine avec la même méthode (source : `prototypes-2026-10-18/p6-egui.rs`, jetable) : **96 Mo de PSS (RSS 106 Mo), un seul processus**, fenêtre visible en 0,34 s, binaire de 27,3 Mo, aucun processus après la fermeture. Soit environ le tiers de la mémoire de la coquille Tauri. Ce que cette mesure **ne dit pas** : la fidélité à la maquette HTML/CSS, l'accessibilité (WCAG 2.2 AA), l'internationalisation, l'effort de refaire les 20 écrans, ni la mémoire avec de vrais écrans. La fenêtre mesurée utilise le moteur graphique par défaut d'`eframe` (non étudié).

**Décision du propriétaire (2026-10-18, ADR 0008)** : conserver Tauri + Svelte pour la V0.1, plafond de 350 Mo de PSS re-mesuré à chaque version, pas d'AppImage en V0.1.

## Chaîne d'outils : ce qui marche, ce qui a surpris

- **`make check` vert** sur le workspace avec Tauri : `fmt`, `clippy -D warnings` (le crate Tauri passe sous les lints pédantiques du workspace), `cargo deny` (aucune bibliothèque réseau sur les cibles Linux et Windows : SEC1 tient pour `tauri` 2.12), `check-rust`, `cargo test`, puis l'interface (`pnpm lint`, `typecheck`, `test`).
- **Le délai de 7 jours n'était pas respecté** par `cargo` : la première résolution a pris **21 crates publiés depuis moins de 7 jours**, dont `tauri` 2.12.4 (2026-10-14). Cargo n'a pas d'équivalent de `minimumReleaseAge`. Méthode de correction : lire la date de publication de chaque version du verrou (API de crates.io, `created_at`), puis `cargo update -p <crate>@<version> --precise <version plus ancienne>` en suivant les dépendances entre crates (trois tours pour que l'ensemble tienne). Résultat : **0 crate sur 407 de moins de 7 jours** (`tauri` 2.12.3, `serde_json` 1.0.152, `tokio` 1.53.2…). Dependabot (délai de 7 jours) couvre les mises à jour futures ; un contrôle automatique du verrou n'existe pas : **à décider**.
- **`cargo audit`** : 0 vulnérabilité sur 407 crates ; 2 avertissements, tous deux par la pile `gtk` 0.18 que `tauri` embarque sous Linux : `proc-macro-error` 1.0.4 non maintenue (RUSTSEC-2024-0370) et `glib` 0.18.5 « unsound » (RUSTSEC-2024-0429, `VariantStrIter`). Notre code n'appelle pas `glib` directement : l'incidence réelle du second n'est **pas vérifiée**. Ils disparaîtront quand `tauri` changera de pile. `--deny warnings` reste désactivé (décision reportée à l'étape 4, tranchée ici : le laisser visible sans bloquer).
- **`pnpm audit`** : aucune vulnérabilité connue.
- **L'AppImage télécharge ses outils à la construction** : `AppRun`, `linuxdeploy` et le greffon `linuxdeploy-plugin-appimage` (version « continuous », donc mouvante) viennent de GitHub et ne sont pas épinglés par empreinte. Pour une publication, c'est un trou de la chaîne de distribution (SEC11) : **à traiter avant la première release** (épingler par empreinte, ou ne publier que `.deb`, `.rpm` et `.msi`).
- **`tauri` cherche `src-tauri/` dans le dossier courant et ses sous-dossiers, pas dans les parents** : l'outil (`@tauri-apps/cli`) est donc dans `apps/desktop/package.json`, pas dans celui de l'interface.
- **SvelteKit 3** : l'alias `$lib` n'existe plus ; les imports internes passent par `#lib/…` (champ `imports` de `package.json`).
- **CI** : `pnpm/action-setup` lisait la version de pnpm dans le `package.json` de l'application ; elle est dans celui de la racine (champ `packageManager`). Corrigé dans `ci.yml`. Le gabarit du kit a le même défaut : à rapporter dans le kit.
- **CI hébergée** : le job Rust a échoué une première fois, `ubuntu-latest` n'ayant pas les paquets de développement de Tauri (`gobject-2.0.pc` introuvable) ; la CI les installe maintenant (liste officielle). Avec cela, les jobs Rust et interface passent sur l'exécuteur GitHub : c'est la première exécution réelle des étapes ajoutées à l'étape 4 (`cargo-audit` et `cargo-deny` installés, toolchain lue dans `rust-toolchain.toml`, pnpm lu dans la racine).
- **Icônes** : générées par `tauri icon` à partir du logo provisoire des maquettes ; les dossiers Android et iOS sont supprimés (pas d'application mobile, PRD §3).

## Non vérifié

- Windows : rien compilé ni mesuré (installeur `.msi`, WebView2).
- La CSP est une proposition stricte (`default-src 'self'`, rien de distant) dont le rendu est vérifié sur cette page seule : chaque écran futur peut demander de l'ajuster.
- La mémoire d'une fenêtre qui affiche de vrais écrans (listes, graphiques) : elle sera plus haute que celle de la coquille.
