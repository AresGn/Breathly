# ADR 0014 : Capacités et politique de contenu de Tauri

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

SEC3 demande que chaque commande Tauri soit enregistrée explicitement et restreinte à la fenêtre principale, parce que l'interface web est traitée comme non fiable. La coquille livrée à l'étape 5 pose une CSP stricte et une capacité vide ; il reste à écrire la règle qui dit où se déclare une commande, comment son accès est restreint, comment on en ajoute une, et ce que la CSP autorise quand les écrans arrivent. Ticket #28 (sécurité §3 ; modèle de menaces, question ouverte n°2).

**Vérifié (2026-10-18)**, détail dans `docs/research/2026-10-18-capacites-tauri.md` (versions du verrou : tauri 2.12.3, tauri-build 2.7.2, tauri-utils 2.10.1) :
- Par défaut, les commandes sont appelables par toutes les fenêtres ; la page « Capabilities » de Tauri recommande `AppManifest::commands` dans `build.rs`.
- Essai dans un projet isolé (`tauri::test`, Linux) : une commande déclarée dans le manifeste et listée dans la capacité de `main` s'exécute ; déclarée dans le manifeste mais absente de la capacité, elle est refusée ; enregistrée dans `invoke_handler` mais absente du manifeste, elle est refusée (« Command not found »). Les permissions s'écrivent `allow-start-disk-analysis` (tirets).
- Sous WebKitGTK, avec la CSP de la coquille **inchangée** et une page Svelte d'essai embarquée dans l'application : une police WOFF2 et une image servies par `'self'` se chargent ; les attributs `style` statique et dynamique de Svelte, la directive `style:` et `element.style` s'appliquent ; `new Function` est bloqué, ainsi que `innerHTML` avec un attribut `style` et `setAttribute('style', …)` (violations `style-src-attr` relevées). `"freezePrototype": true` ne change rien à ce rendu.
- `tauri-utils` expose `devCsp` et `freezePrototype` (désactivé par défaut). La documentation CSP de Tauri dit que Tauri ajoute nonces et hashes aux scripts et styles locaux à la compilation.

**Non vérifié.** Windows (WebView2) : l'origine de l'application y est `http://tauri.localhost` (`useHttpsScheme`, code source lu), contrairement à `tauri://localhost` sous Linux ; le refus observé avec `http://tauri.localhost` dans l'essai vient de l'environnement de test sous Linux et ne prouve rien pour Windows ; un écran réel de Breathly sous la CSP ; `devCsp` avec le serveur de développement ; `freezePrototype` avec les appels `invoke` ; l'appel depuis une fenêtre de label différent ; les plugins face aux capacités.

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
- La capacité `apps/desktop/src-tauri/capabilities/main.json` (identifiant `main`, `windows: ["main"]`, aucune section `remote`) **remplace** `default.json` (supprimé : tout fichier de `capabilities/` est activé, donc il ne doit en rester qu'un). Elle liste une permission `allow-<commande en tirets>` pour **chaque** commande de la liste, et les seules permissions de plugin utiles.
- **Ajouter une commande**, dans la PR qui la crée : son nom entre dans la liste, sa permission dans la capacité, la fonction dans `generate_handler!`. Une PR qui n'ajoute que l'un des trois échoue au test.
- **Test TS-05**, livré par le ticket #34 : des contrôles de fichiers (la liste de commandes, la capacité `main` et le gestionnaire de commandes correspondent ; aucune permission ne cite une commande absente de la liste ; il n'existe qu'un fichier de capacité et qu'une fenêtre, `main`) et des appels réels avec `tauri::test` depuis l'origine locale du test : chaque commande de la liste est acceptée, un nom absent de la liste est refusé. Les capacités étant compilées avec l'application, le test ne retire pas de permission : le cas « dans le manifeste mais pas dans la capacité » est couvert par la comparaison des fichiers. Le test TS-03 (aucune commande ne reçoit un chemin à supprimer) lit la même liste.
- Une commande ne reçoit jamais un chemin à supprimer (SEC3). L'exception unique, écrite dans le test, est `set_settings`, qui transporte la liste d'exclusion (des chemins à protéger).

### Politique de contenu

- La CSP de production reste celle de la coquille : `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; frame-ancestors 'none'`. Aucun `unsafe-inline`, aucun `unsafe-eval`, aucune source distante.
- **Styles** : l'interface applique ses styles par Svelte (attribut `style`, directive `style:`) ou par `element.style` ; elle ne pose **aucun** attribut `style` par `innerHTML` ni par `setAttribute('style', …)`, bloqués par la CSP (vérifié). `{@html}` reste interdit (`svelte/no-at-html-tags`).
- Les polices WOFF2 sont servies par `font-src 'self'` : Vite est réglé pour ne pas les inliner en `data:` (ticket #46, dont le critère « aucune URL externe » se vérifie aussi sur le résultat de la construction).
- Une CSP **`devCsp` séparée** autorise le serveur de développement (HMR de Vite) sans toucher à la production. Son contenu exact est fixé par essai dans la PR du ticket #34.
- `freezePrototype: true` est **essayé** avec Svelte dans la PR du ticket #34 et gardé s'il ne casse rien ; si une dépendance de l'interface casse, le résultat est écrit dans une note datée sur cet ADR.
- Tout assouplissement de la CSP de production se fait par une PR qui nomme l'écran qui l'exige, la source ajoutée et la raison, et ajoute une note datée sur cet ADR.

## Conséquences

- Le ticket #34 écrit la liste des commandes (`command_list.rs`), le `build.rs`, la capacité `main` (en supprimant `default.json`), le test TS-05, la `devCsp` (essai avec le serveur de Vite) et l'essai de `freezePrototype` avec `invoke`, et ajoute à l'essai un composant avec un attribut `style=` et une directive `style:`. Son corps est complété après la fusion de cette ADR.
- Le ticket #134 (installation et première ouverture) ajoute la vérification sous Windows : les commandes autorisées s'exécutent depuis l'origine `http://tauri.localhost`, sans ouvrir la capacité à une origine distante (aucune section `remote`).
- Chaque ticket de contrat (#37, #61, #86, #113) ajoute ses commandes à la liste dans sa PR.
- Le défaut est le refus : une commande oubliée dans le manifeste ou la capacité ne s'exécute pas (vérifié par essai). Le risque restant est l'inverse : une permission de plugin trop large, relue en PR.
- Les essais sous WebView2 et avec un écran réel restent à faire ; ils sont suivis par le ticket #134 et par les tickets d'écran.

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Le design 0002 §6 (« chaque commande est donc listée dans les capacités et restreinte à la fenêtre principale ») est mis en œuvre sans changement de texte.
