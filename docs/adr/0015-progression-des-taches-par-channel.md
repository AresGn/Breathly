# ADR 0015 : Progression des tâches par Channel

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le diagnostic, l'analyse du disque et le nettoyage durent plusieurs secondes : l'interface affiche leur progression et un bouton Annuler (US-001.3, écrans H2 et N3). L'ADR 0013 a décidé que le cœur émet l'avancement dans un canal `std::sync::mpsc` et que le registre de tâches le relaie vers l'interface « par l'API que décidera le ticket #26 ». Design 0002 §6 et §9 point 5 : les noms exacts de l'API de Tauri étaient « non vérifiés ».

**Vérifié (2026-10-18)**, détail dans `docs/research/2026-10-18-progression-tauri.md` :
- Documentation de Tauri v2 (https://v2.tauri.app/develop/calling-frontend/) : les événements n'ont pas de typage fort et ne sont pas faits pour un débit élevé ; les canaux sont « designed to be fast and deliver ordered data » et recommandés pour le streaming ordonné.
- Code source de `tauri` 2.12.3 : chaque message d'un Channel porte son index et la partie JavaScript remet les messages dans l'ordre ; le Channel se ferme (`end`) quand il est détruit.
- Essai dans une fenêtre réelle (WebKitGTK, build de debug, capacité qui ne liste que les commandes de l'application) : un `Channel` passé en argument d'une commande et alimenté par un `std::thread` livre 20 002 messages sur 20 002, dans l'ordre, en 1,5 s ; `event.listen` est refusé tant que la capacité n'a pas de permission d'événement.

**Non vérifié.** Windows (WebView2) ; build `release` ; rechargement de la fenêtre en cours de tâche ; comportement de `send` quand la fenêtre est fermée.

## Options

1. **Un Channel par tâche, passé au lancement (retenue).** Typé, ordonné, sans permission d'événement, fermé à la fin de la tâche.
2. **Événements Tauri avec l'identifiant de tâche.** Diffusés à qui écoute ; non typés ; sans ordre documenté ; exigent `core:event:allow-listen` (et `unlisten`) dans la capacité de `main`, donc une surface de plus pour SEC3.
3. **Sondage : `get_task_progress` toutes les 250 ms.** Très simple, mais un décalage allant jusqu'à 250 ms, une commande de plus à inventorier (TS-05) et du travail de l'interface quand rien ne change.

## Décision

**Option 1.**

- Chaque commande qui lance une tâche (`start_diagnostic`, `start_disk_analysis`, `start_cleanup`) reçoit un argument `on_progress: Channel<ProgressEvent>` créé par l'interface au moment de l'appel. Le registre de tâches (ADR 0013) en garde l'extrémité d'envoi.
- `ProgressEvent` est une **union fermée** (balise `kind`), générée vers TypeScript par le mécanisme de l'ADR 0016 :
  - `started` ;
  - `step { done, total, step }` : `done` est croissant ; `total` est absent quand il est inconnu (l'interface affiche alors une progression indéterminée) ; `step` est une **clé de traduction** de l'étape en cours, jamais un texte ni un chemin (SEC9) ;
  - `finished`, `cancelled` ;
  - `failed { error }` : l'erreur typée du contrat (code stable, clé de traduction, paramètres).
- **Une tâche envoie exactement un message terminal** (`finished`, `cancelled` ou `failed`), puis le registre détruit le Channel : l'interface reçoit alors la fin du flux. Une tâche qui panique envoie `failed` (ADR 0013).
- **Le relais coalesce** : il garde un fil de relais entre le canal `mpsc` du cœur et le Channel, transmet toujours `started` et le message terminal, et ne transmet que le dernier `step` à intervalle d'au moins 50 ms (environ 20 mises à jour par seconde ; **valeur de départ non vérifiée**, l'essai a mesuré plus de 13 000 messages par seconde en debug, ce qui n'est pas le goulot). Les messages ne sont jamais réordonnés.
- Le **résultat** d'une tâche se lit ensuite par `get_task_result` (ticket #61) ; le Channel ne porte aucun résultat.
- **Annulation** : `cancel_task(task_id)` (ticket #36), indépendante du Channel.
- Une erreur d'envoi (fenêtre fermée) est journalisée avec un code (SEC9) et **n'interrompt pas la tâche** : un nettoyage en cours se termine proprement (ADR 0013).
- L'interface garde le dernier message reçu dans un module d'état (`.svelte.ts`) : revenir sur l'écran H2 ou N3 après une navigation ne perd pas la progression, parce que le Channel vit dans l'objet JavaScript, pas dans l'écran.
- Aucune permission d'événement n'est ajoutée à la capacité de `main` ; l'interface n'appelle ni `listen` ni `emit`.

## Conséquences

- Le ticket #36 (registre de tâches) écrit le fil de relais, la coalescence, le message terminal et le test avec une fausse tâche longue (progression croissante, un seul message terminal, tâche qui panique → `failed`).
- Les tickets de contrat (#61, #86, #113) ajoutent l'argument `on_progress` aux commandes de lancement et le type `ProgressEvent` aux types générés.
- Les écrans H2 et N3 lisent le dernier message du module d'état ; leurs tests utilisent un faux flux.
- Le design 0002 §6 (« noms exacts de l'API Tauri : non vérifiés ») et §9 point 5 passent à « décidé » dans la PR du ticket #36.
- Si Windows se comporte autrement (ordre, perte), c'est un défaut à corriger dans la PR de validation (ticket #134), pas une raison de revenir aux événements sans nouvelle décision.

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Le cœur ne connaît ni `Channel` ni Tauri : il émet un type de progression de la bibliothèque standard ; la conversion vers `ProgressEvent` se fait dans `apps/desktop/src-tauri`.
