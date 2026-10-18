# Capacités et CSP de Tauri : essai du 2026-10-18

Alimente l'ADR 0014 (ticket #28).

## Question

Comment restreindre chaque commande Tauri de Breathly à la fenêtre principale (SEC3), et quelle politique de contenu pour les écrans à venir ?

## Sources lues (2026-10-18)

- https://v2.tauri.app/security/capabilities/ : une capacité cible des fenêtres par leur **label** (pas leur titre) ; sans restriction, les commandes sont « allowed to be used by all the windows and webviews of the app » ; la page recommande `tauri_build::AppManifest::new().commands(...)` dans `build.rs` pour définir les commandes autorisées ; tout fichier de `src-tauri/capabilities` est activé automatiquement.
- https://v2.tauri.app/security/csp/ : la CSP n'est active que si elle est définie dans la configuration ; Tauri ajoute lui-même nonces et hashes aux scripts et styles locaux à la compilation ; ne charger que des ressources de confiance ; `connect-src` de l'exemple : `ipc:` et `http://ipc.localhost`.
- Code source de `tauri-build` 2.7.3 : `AppManifest::commands` génère des permissions `allow-$command` et `deny-$command`.
- Code source de `tauri-utils` 2.10.2 : la configuration de sécurité expose `devCsp` (CSP de développement séparée), `freezePrototype` (désactivé par défaut) et `dangerousDisableAssetCspModification`.

## Essai

Projet isolé (hors dépôt, copie du `Cargo.lock` du dépôt, tauri 2.12.x), dont les sources sont dans le dossier `capacites-tauri-2026-10-18/` de cette note (fichiers `.txt`). `build.rs` déclare deux commandes dans le manifeste (`start_disk_analysis`, `cancel_task`) ; la capacité `main` ne liste que `allow-start-disk-analysis` ; trois commandes sont enregistrées dans `invoke_handler`, dont une absente du manifeste. Appels faits avec `tauri::test` depuis la fenêtre `main`, origine locale `tauri://localhost` :

| Commande | Manifeste | Capacité | Résultat |
|---|---|---|---|
| `start_disk_analysis` | oui | oui | exécutée (`Ok(1)`) |
| `cancel_task` | oui | non | refusée : « not allowed. Permissions associated with this command: allow-cancel-task » |
| `pas_dans_le_manifeste` | non | non | refusée : « not allowed. Command not found » |

Avec l'origine `http://tauri.localhost` (considérée comme distante par la capacité), même la commande autorisée est refusée : « allowed on: [windows: "main", URL: local] ». La permission s'écrit avec des tirets (`allow-start-disk-analysis`) pour une commande en snake_case.

## Non vérifié

- Une fenêtre de label différent (la capacité cite `windows: ["main"]` ; le message d'erreur le confirme, l'appel depuis un autre label n'a pas été essayé).
- WebView2 (Windows) et un écran Svelte réel sous la CSP de la coquille.
- `freezePrototype` avec Svelte ; `devCsp` avec le serveur de développement de Vite.
- Le comportement des plugins (dialogue) face aux capacités.
