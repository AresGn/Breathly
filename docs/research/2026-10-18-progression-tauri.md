# Progression d'une tâche vers l'interface : Channel et événements de Tauri, essai du 2026-10-18

Alimente l'ADR 0015 (ticket #26).

## Question

Comment le cœur envoie-t-il la progression d'une tâche longue à l'interface (US-001.3), de façon typée, ordonnée, sans élargir les capacités de la fenêtre principale (SEC3) ?

## Sources lues (2026-10-18)

- https://v2.tauri.app/develop/calling-frontend/ :
  - événements : « Global events are delivered to all listeners » ; « events have no strong type support » ; « event payloads are always JSON strings making them not suitable for bigger messages » ; « The event system is not designed for low latency or high throughput situations » ; la page dit qu'il n'y a « no support of the capabilities system to fine grain control event data and channels » ;
  - canaux : la commande reçoit `Channel<T>`, le frontend crée `new Channel<T>()` et le passe à `invoke`, Rust appelle `send`, le frontend définit `onmessage` ; « Channels are designed to be fast and deliver ordered data » ; `T` est sérialisable avec serde (l'exemple est une énumération Rust, équivalent TypeScript : union de types) ; la page recommande les canaux pour le streaming et les flux ordonnés.
- Code source de `tauri` 2.12.3 (`src/ipc/channel.rs`, version du verrou) : « The channel automatically orders the messages: every message carries the index it was sent with, and the JavaScript side buffers out-of-order messages until the missing ones arrive » ; à la destruction du `Channel`, le frontend reçoit `{ end: true, index }` ; `Channel::send(&self, data)` exige `TSend: IpcResponse` (sérialisable) ; le crate a une fonction `specta`.

## Essai

Projet isolé (hors dépôt, `Cargo.lock` du dépôt : tauri 2.12.3), sources dans `progression-tauri-2026-10-18/` (fichiers `.txt`). Application réelle (`--features tauri/custom-protocol`, build de debug) lancée sous Linux (WebKitGTK), CSP stricte (`default-src 'self'; script-src 'self'; connect-src ipc: http://ipc.localhost`), `withGlobalTauri`. La capacité `default` ne liste que `allow-start-disk-analysis` et `allow-emit-test` (aucune permission d'événement).

1. **Compilation** : un `Channel<Progress>` se déplace dans un `std::thread` (il est `Send`) ; une tâche sur un fil et un relais sur un autre fil (`std::sync::mpsc`) compilent.
2. **Canal** : `start_disk_analysis(on_progress: Channel<Progress>, n)` envoie `Started`, `n` messages `Step { done, total }`, `Finished` depuis un fil standard.

| n | Messages reçus | Ordre | Message final | Durée |
|---|---|---|---|---|
| 20 | 22 sur 22 | respecté | reçu | 12 ms |
| 20 000 | 20 002 sur 20 002 | respecté | reçu | 1 533 ms |

3. **Événements** : `window.__TAURI__.event.listen('progress', …)` avec la même capacité est **refusé** : « event.listen not allowed. Permissions associated with … ». Utiliser les événements imposerait d'ajouter une permission d'événement à la capacité de `main`. La mesure du débit des événements n'a donc pas été faite.

## Non vérifié

- WebView2 (Windows) : les Channel y passent par un autre mécanisme d'exécution (`eval` ou `fetch`, selon la taille, d'après les commentaires du code source) ; ordre et perte non essayés.
- Build `release` ; machine lente ; un rechargement de la fenêtre pendant une tâche (le Channel est lié à l'appel `invoke`).
- Le débit des événements ; le coût mémoire d'un Channel de longue durée.
- Un Channel dont la fenêtre est fermée avant la fin : `send` renvoie une erreur (comportement non essayé).
