# Capacités et CSP de Tauri : essai du 2026-10-18

Alimente l'ADR 0014 (ticket #28).

## Question

Comment restreindre chaque commande Tauri de Breathly à la fenêtre principale (SEC3), et quelle politique de contenu pour les écrans à venir ?

## Sources lues (2026-10-18)

- https://v2.tauri.app/security/capabilities/ : une capacité cible des fenêtres par leur **label** (pas leur titre) ; sans restriction, les commandes sont « allowed to be used by all the windows and webviews of the app » ; la page recommande `tauri_build::AppManifest::new().commands(...)` dans `build.rs` pour définir les commandes autorisées ; tout fichier de `src-tauri/capabilities` est activé automatiquement.
- https://v2.tauri.app/security/csp/ : la CSP n'est active que si elle est définie dans la configuration ; Tauri ajoute lui-même nonces et hashes aux scripts et styles locaux à la compilation ; ne charger que des ressources de confiance ; `connect-src` de l'exemple : `ipc:` et `http://ipc.localhost`.
- Versions du verrou du dépôt (`Cargo.lock`) : tauri 2.12.3, tauri-build 2.7.2, tauri-utils 2.10.1, wry 0.57.0, webkit2gtk 2.0.2.
- Code source de `tauri-build` (lu en 2.7.3 ; le passage sur `AppManifest` est identique en 2.7.2, comparé) : `AppManifest::commands` génère des permissions `allow-$command` et `deny-$command`.
- Code source de `tauri-utils` 2.10.1 : la configuration de sécurité expose `devCsp` (CSP de développement séparée), `freezePrototype` (désactivé par défaut) et `dangerousDisableAssetCspModification` ; `useHttpsScheme` : sous Windows et Android, les protocoles personnalisés sont `http://<scheme>.localhost` par défaut (`https` si l'option est vraie), alors que macOS et Linux utilisent `<scheme>://localhost`.

## Essai

Projet isolé (hors dépôt, copie du `Cargo.lock` du dépôt : tauri 2.12.3, tauri-build 2.7.2), dont les sources sont dans le dossier `capacites-tauri-2026-10-18/` de cette note (fichiers `.txt`). `build.rs` déclare deux commandes dans le manifeste (`start_disk_analysis`, `cancel_task`) ; la capacité `main` ne liste que `allow-start-disk-analysis` ; trois commandes sont enregistrées dans `invoke_handler`, dont une absente du manifeste. Appels faits avec `tauri::test` depuis la fenêtre `main`, origine locale `tauri://localhost` :

| Commande | Manifeste | Capacité | Résultat |
|---|---|---|---|
| `start_disk_analysis` | oui | oui | exécutée (`Ok(1)`) |
| `cancel_task` | oui | non | refusée : « not allowed. Permissions associated with this command: allow-cancel-task » |
| `pas_dans_le_manifeste` | non | non | refusée : « not allowed. Command not found » |

Avec l'origine `http://tauri.localhost`, la commande autorisée est refusée par l'environnement de test sous Linux : « allowed on: [windows: "main", URL: local] ». **Ce n'est pas une preuve que cette origine est refusée sur Windows** : `http://tauri.localhost` y est l'origine normale de l'application (`useHttpsScheme`, ci-dessus), alors que sous Linux l'origine locale est `tauri://localhost`. Le comportement sous Windows reste à vérifier (voir « Non vérifié »). La permission s'écrit avec des tirets (`allow-start-disk-analysis`) pour une commande en snake_case.

## Essai 2 : la CSP de la coquille sous WebKitGTK

Page d'essai Svelte (source : `capacites-tauri-2026-10-18/essai-csp-page.svelte.txt`) construite avec l'interface du dépôt, dans une copie jetable (`git worktree`), embarquée dans l'application (`--features tauri/custom-protocol`), lancée sous Linux (WebKitGTK 2.0.2 des liaisons du verrou) avec la CSP de `tauri.conf.json` **inchangée**. Une police WOFF2 (`font-src 'self'`), une image PNG (`img-src 'self'`) et quatre barres dont la largeur vient d'un attribut `style` statique, d'un attribut `style` dynamique, de la directive `style:` de Svelte et de `element.style` (CSSOM). Contrôles négatifs : `new Function`, `innerHTML` avec attribut `style`, `setAttribute('style', …)` ; les violations sont relevées par l'événement `securitypolicyviolation`.

| Élément | Résultat |
|---|---|
| Police WOFF2 depuis `/fonts/essai.woff2` | chargée (`document.fonts` : `loaded`) et affichée |
| Image PNG | chargée (128 px) |
| Attribut `style` statique de Svelte | appliqué (200 px) |
| Attribut `style` dynamique de Svelte | appliqué (200 px) |
| Directive `style:width` de Svelte | appliquée (200 px) |
| `element.style` (CSSOM) | appliqué (200 px) |
| `new Function('return 1')` | bloqué (`EvalError`) : la CSP est bien active |
| `innerHTML` avec un attribut `style` | **bloqué** (largeur 640 px au lieu de 200) |
| `setAttribute('style', …)` | **bloqué** (largeur 640 px au lieu de 200) |
| Violations signalées | `script-src (eval)`, deux `style-src-attr (inline)` ; aucune pour les quatre barres de Svelte |

Même page avec `"freezePrototype": true` : rendu et résultats identiques, aucune erreur. La page n'appelle pas l'API de Tauri : l'effet de `freezePrototype` sur `invoke` n'est pas essayé.

La police de l'essai (KaTeX_Script, licence OFL) n'est pas copiée dans le dépôt.

## Non vérifié

- **Windows (WebView2)** : l'origine de l'application y est `http://tauri.localhost` ; les commandes autorisées doivent s'y exécuter. À essayer sur la machine de validation.
- Une fenêtre de label différent (la capacité cite `windows: ["main"]` ; le message d'erreur le confirme, l'appel depuis un autre label n'a pas été essayé).
- Un écran réel de Breathly sous la CSP (seule une page d'essai Svelte l'a été) ; `devCsp` avec le serveur de développement de Vite ; `freezePrototype` avec les appels `invoke`.
- Le comportement des plugins (dialogue) face aux capacités.
- Retirer une permission de la capacité dans un même test : les capacités sont compilées avec l'application, l'essai n'a utilisé qu'une capacité fixe.
