# ADR 0014 : Capacités et politique de contenu de Tauri

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

SEC3 demande que chaque commande Tauri soit enregistrée explicitement et restreinte à la fenêtre principale, parce que l'interface web est traitée comme non fiable. La coquille livrée à l'étape 5 pose une CSP stricte et une capacité vide ; il reste à écrire la règle qui dit où se déclare une commande, comment son accès est restreint, comment on en ajoute une, et ce que la CSP autorise quand les écrans arrivent. Ticket #28 (sécurité §3 ; modèle de menaces, question ouverte n°2).

**Vérifié (2026-10-18)**, détail dans `docs/research/2026-10-18-capacites-tauri.md` :
- Par défaut, les commandes sont appelables par toutes les fenêtres ; la page « Capabilities » de Tauri recommande `AppManifest::commands` dans `build.rs`.
- Essai dans un projet isolé (tauri 2.12.x, `tauri::test`) : une commande déclarée dans le manifeste et listée dans la capacité de `main` s'exécute ; déclarée dans le manifeste mais absente de la capacité, elle est refusée ; enregistrée dans `invoke_handler` mais absente du manifeste, elle est refusée (« Command not found »). Une origine non locale est refusée même pour une commande autorisée.
- Les permissions d'une commande `start_disk_analysis` s'écrivent `allow-start-disk-analysis`.
- La documentation CSP de Tauri dit que Tauri ajoute nonces et hashes aux scripts et styles locaux à la compilation ; `tauri-utils` expose `devCsp` et `freezePrototype` (désactivé par défaut).

**Non vérifié.** Un écran Svelte réel sous la CSP de la coquille ; WebView2 ; `freezePrototype` avec Svelte ; `devCsp` avec le serveur de développement ; l'appel depuis une fenêtre de label différent ; les plugins face aux capacités.

## Options

**Restriction des commandes.**
1. Manifeste d'application + capacité de `main` + test de cohérence (retenue).
2. Capacité générée par `build.rs` : impossible d'oublier une permission, mais un fichier de sécurité produit par du code de construction, moins lisible en relecture ; non essayé.
3. Capacité vide, sans manifeste (défaut de Tauri) : toute commande reste appelable depuis toute fenêtre ; contredit SEC3.

**Politique de contenu.**
1. CSP stricte inchangée, `devCsp` séparée, essai de `freezePrototype` (retenue).
2. Ajouter `style-src 'unsafe-inline'` : ouvre l'injection de styles.
3. Assouplir au cas par cas sans règle écrite d'avance.

## Décision

### Commandes

- Breathly n'a **qu'une fenêtre**, de label `main`. Aucune autre fenêtre ni webview n'est créée en V0.1.
- La **liste des commandes** est un fichier Rust unique (`apps/desktop/src-tauri/src/command_list.rs`, une constante de noms en snake_case). `build.rs` l'inclut (`include!`) et la passe à `AppManifest::new().commands(...)` ; le code de l'application la réutilise.
- La capacité `apps/desktop/src-tauri/capabilities/main.json` (identifiant `main`, `windows: ["main"]`, aucune section `remote`) liste une permission `allow-<commande en tirets>` pour **chaque** commande de la liste, et les seules permissions de plugin utiles.
- **Ajouter une commande**, dans la PR qui la crée : son nom entre dans la liste, sa permission dans la capacité, la fonction dans `generate_handler!`. Une PR qui n'ajoute que l'un des trois échoue au test.
- **Test TS-05**, livré par le ticket #34 : il vérifie que la liste, la capacité et le gestionnaire de commandes correspondent (chaque commande de la liste est appelable depuis `main` avec l'origine locale et rejetée sans sa permission), qu'aucune permission ne cite une commande absente de la liste, et qu'il n'existe aucune autre fenêtre que `main`. Le test TS-03 (aucune commande ne reçoit un chemin à supprimer) lit la même liste.
- Une commande ne reçoit jamais un chemin à supprimer (SEC3). L'exception unique, écrite dans le test, est `set_settings`, qui transporte la liste d'exclusion (des chemins à protéger).

### Politique de contenu

- La CSP de production reste celle de la coquille : `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; frame-ancestors 'none'`. Aucun `unsafe-inline`, aucun `unsafe-eval`, aucune source distante.
- Les polices WOFF2 sont servies par `font-src 'self'` : Vite est réglé pour ne pas les inliner en `data:` (ticket #46).
- Une CSP **`devCsp` séparée** autorise le serveur de développement (HMR de Vite) sans toucher à la production. Son contenu exact est fixé par essai dans la PR du ticket #34.
- `freezePrototype: true` est **essayé** avec Svelte dans la PR du ticket #34 et gardé s'il ne casse rien ; si une dépendance de l'interface casse, le résultat est écrit dans une note datée sur cet ADR.
- Tout assouplissement de la CSP de production se fait par une PR qui nomme l'écran qui l'exige, la source ajoutée et la raison, et ajoute une note datée sur cet ADR.

## Conséquences

- Le ticket #34 écrit la liste des commandes, le `build.rs`, la capacité `main`, le test TS-05, la `devCsp` et l'essai de `freezePrototype`.
- Chaque ticket de contrat (#37, #61, #86, #113) ajoute ses commandes à la liste dans sa PR.
- Le défaut est le refus : une commande oubliée dans le manifeste ou la capacité ne s'exécute pas (vérifié par essai). Le risque restant est l'inverse : une permission de plugin trop large, relue en PR.
- Les essais sous WebView2 et avec un écran réel restent à faire ; ils sont suivis par les tickets de validation de la vague M4 et par les tickets d'écran.

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Le design 0002 §6 (« chaque commande est donc listée dans les capacités et restreinte à la fenêtre principale ») est mis en œuvre sans changement de texte.
