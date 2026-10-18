# ADR 0013 : Exécution des tâches longues : fils standard

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le diagnostic, l'analyse du disque et le nettoyage durent de quelques secondes à une minute. Il faut dire comment le cœur les exécute, comment elles s'annulent (design 0001 §1, US-001.3) et comment leur progression arrive à l'interface (ticket #26 pour l'API de Tauri). Point ouvert n°4 du design 0002 §9 ; ticket #25. Le choix fixe la forme des traits de `breathly-platform`, du registre de tâches et du moteur d'analyse.

**Vérifié (2026-10-18).**
- Aucune des bibliothèques retenues par l'ADR 0012 n'a d'API asynchrone : `sysinfo` 0.39.6, `jwalk` 0.9.0, `trash` 5.2.9, `open` 5.4.4, `dirs` 7.0.0 ne contiennent ni `async fn` ni `.await` (code source lu). `jwalk` parallélise avec son propre pool de fils (`rayon`).
- Documentation de Tauri v2, « Calling Rust from the frontend » (https://v2.tauri.app/develop/calling-rust/, lue le 2026-10-18) : « Commands without the async keyword are executed on the main thread » sauf `#[tauri::command(async)]` ; une commande asynchrone « is executed on a separate async task » ; les canaux (Channel) sont « the recommended mechanism for streaming data » et conseillés pour une livraison ordonnée. La page ne parle ni de `spawn_blocking` ni de fils standard.
- `tokio` 1.53.2 est déjà dans `Cargo.lock` par Tauri, et `tauri::async_runtime::spawn_blocking` existe (code source de tauri 2.12.3).

**Non vérifié.** Le comportement exact des Channel sous charge et leur ordre (ticket #26) ; la durée réelle de ces commandes sur une grosse corbeille ; la latence d'annulation sur un disque lent ; le comportement sous Windows.

## Options

1. **Cœur synchrone, un fil standard par tâche.** Les traits et fonctions du cœur sont bloquants. Une tâche est un fil nommé (`std::thread`), l'annulation passe par un jeton atomique vérifié entre deux éléments, la progression par un canal de la bibliothèque standard relayé vers l'interface.
2. **Cœur synchrone, tâches lancées par `spawn_blocking` de Tauri.** Même cœur, mais le registre de tâches confie les fils au pool bloquant de tokio. Moins de code de fils, mais le registre dépend de l'environnement de Tauri pour ses tests.
3. **Cœur asynchrone (tokio, traits async).** Chaque trait de `platform` devient asynchrone. Comme aucune bibliothèque retenue n'est asynchrone, chaque appel passerait quand même par un fil bloquant : plus de complexité (traits asynchrones, objets dynamiques en boîtes) et une dépendance de plus dans `breathly-platform`, sans gain.

## Décision

**Option 1.**

- Tous les traits de `breathly-platform` (`Clock`, `TrashProvider`, `ProcessInspector`, `DiskInspector`, `TemperatureReader`, `SystemInspector`) et toutes les fonctions publiques des crates du cœur sont **synchrones**. Aucun crate du cœur ne dépend de `tokio` ni d'un crate asynchrone.
- Une tâche (diagnostic, analyse, nettoyage) s'exécute sur **un fil nommé** lancé par le registre de tâches de `apps/desktop/src-tauri`. Il y a au plus une tâche par type à la fois (US-001.4), donc au plus trois fils de tâche, plus le pool de `jwalk` dont la borne vient de la mesure du ticket #48.
- La commande Tauri qui lance une tâche **rend la main tout de suite** avec l'identifiant de tâche ; elle ne fait aucun travail bloquant (une commande synchrone tourne sur le fil principal).
- **Aucune commande ne bloque le fil principal.** Les opérations longues (diagnostic, analyse du disque, nettoyage) sont des tâches du registre. Toute autre commande qui touche au disque ou à la corbeille (`simulate_cleanup`, `preview_purge_trash`, `purge_trash`, `restore_entry`, `list_history`, `erase_unrecoverable_entries`, `open_folder`) est déclarée `#[tauri::command(async)]`, pour ne jamais s'exécuter sur le fil principal. Si l'une d'elles dépasse quelques secondes en pratique (par exemple le vidage d'une très grosse corbeille), son ticket de contrat la fait passer par le registre de tâches, avec progression.
- **Une tâche qui s'arrête sur une erreur (`panic`) devient « échouée ».** Le registre intercepte l'arrêt du fil (événement `fatal` du design 0001 §1), libère le verrou « une tâche par type » et envoie une erreur typée à l'interface. Sans cela, la tâche resterait « en cours » et le bouton « Analyser mon PC » désactivé jusqu'au redémarrage (US-001.4). Cela suppose que le profil de compilation garde `panic = "unwind"` (le défaut ; aucun `[profile]` du dépôt ne le change aujourd'hui) : un test du registre le vérifie avec une fausse tâche qui panique.
- **Annulation coopérative.** Le cœur reçoit un jeton d'annulation (`Arc<AtomicBool>` dans un type dédié) et le consulte entre deux éléments traités ; il ne tue jamais un fil. Une tâche annulée jette son résultat partiel et ne modifie aucun fichier (design 0001 §1). Pour le nettoyage, l'arrêt demandé laisse l'élément en cours se terminer (US-036.3).
- **Progression.** Le cœur émet des mesures croissantes dans un canal `std::sync::mpsc` ; le registre les relaie vers l'interface par l'API que décidera le ticket #26. Le format du message reste celui de #26.
- Le registre de tâches ne dépend d'aucun type de Tauri dans sa logique (lancement, états, annulation, refus d'un second lancement) : il se teste avec une fausse tâche longue, sans fenêtre.

## Conséquences

- Les faux de test (`Clock`, `TrashProvider`…) sont de simples valeurs, sans exécuteur asynchrone.
- Le ticket #26 (progression) choisit entre Channel et événements sur ce modèle de fils ; le ticket #27 (types) n'est pas touché.
- Si un besoin asynchrone réel apparaît (service Breathly en V0.2, socket local de l'ADR 0007), il vit dans le crate qui en a besoin (`breathly-daemon`), pas dans le cœur des traits de `platform`. Cet ADR serait alors complété d'une note datée, pas réécrit.
- Les fils nommés se voient dans les outils de diagnostic ; chaque tâche a un nom (`breathly-analyse-disque`, par exemple).
- Coût mémoire : un fil de pile standard par tâche, sans pool de plus ; mesure de l'ensemble au ticket #48.

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Le design 0002 §9 point 4 passe à « décidé » dans la PR du ticket du registre de tâches. Les tickets de traits (#30), du moteur d'analyse (#56) et du registre de tâches (#36) sont débloqués par cette décision.
