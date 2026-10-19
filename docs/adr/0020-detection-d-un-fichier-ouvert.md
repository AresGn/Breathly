# ADR 0020 : Détection d'un fichier ouvert

- **Statut** : accepté (Linux et Windows 11 essayés ; conteneurs, Windows 10 et Restart Manager non vérifiés)
- **Date** : 2026-10-19
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le garde-fou S5 (invariant 3 d'`AGENTS.md`, US-034.1) interdit de supprimer un fichier ouvert par un logiciel, et le nettoyage revérifie les garde-fous pour chaque élément au moment d'agir (US-036.4). Il fallait établir comment le savoir sous Linux et sous Windows **sans lancer de programme externe** (`CODING.md` §6), avec les limites de chaque méthode et ce que fait Breathly quand il ne sait pas. Ticket #70. Résultats et sources : `docs/research/2026-10-19-fichiers-ouverts.md`.

**Mesuré.**
- **Linux** : les descripteurs de `/proc/<pid>/fd` (identité dev et inode) détectent un fichier ouvert en lecture-écriture, en lecture seule ou verrouillé par `flock` ; ils **ne voient pas** un fichier cartographié en mémoire dont le descripteur est fermé, que `/proc/<pid>/maps` détecte. `File::try_lock_shared` ne voit que les verrous `flock`. Instantané des descripteurs : 11 à 12 ms ; avec les cartes : 104 à 115 ms ; 1 000 consultations : 21 à 34 µs. Sur 342 processus, 175 sont illisibles : environ 171 d'un autre compte et 3 du compte courant.
- **Windows 11 (poste client)** : une ouverture avec accès `DELETE` (partage total) échoue avec le code 32 dans les trois cas où `remove_file` échoue (exclusif, lecture-écriture partagée sans partage de suppression, lecture seule) et réussit dans le seul cas où `remove_file` réussit (autre processus avec partage de suppression). La sonde **prédit** donc la suppression ; elle ne voit pas un programme qui a ouvert le fichier avec partage de suppression.

**Non vérifié.** Windows 10 et disques réseau ; le Restart Manager de Windows ; Linux dans un conteneur, snap, Flatpak, avec `hidepid` ou sans `/proc` ; les dossiers contenant plusieurs milliers de fichiers ouverts ; le coût d'un nettoyage réel ; l'effet, sur un logiciel, de la suppression d'un fichier qu'il tient avec partage de suppression.

## Options

1. **Linux : instantané de `/proc` (descripteurs et cartes), Windows : sonde d'ouverture avec accès `DELETE` (retenue).**
2. Refuser par prudence dès qu'un processus du compte est illisible : tout nettoyage serait refusé ici (3 processus protégés).
3. Ne détecter que sous Windows : l'invariant 3 ne serait pas tenu sous Linux, où la suppression d'un fichier ouvert réussit toujours.

## Décision

**Option 1**, avec les règles suivantes.

- **Un port dans `breathly-platform`** : un trait `OpenFileInspector` (avec son faux de test), qui répond, pour un élément (fichier ou dossier), « libre », « utilisé par ces logiciels » ou « vérification impossible ». La liste « Ports à implémenter » d'`AGENTS.md` et le design 0002 §5 l'ajoutent dans la PR du ticket #73.
- **Linux.** Pour **chaque élément, au moment d'agir**, un instantané lit les descripteurs de chaque `/proc/<pid>/fd` lisible et les cartes de chaque `/proc/<pid>/maps` (dev, inode, chemin) ; les processus de Breathly sont exclus. Un **fichier** est utilisé si son identité (dev, inode) figure dans l'instantané ; un **dossier** est utilisé par un fichier si un chemin de l'instantané est sous lui (descripteurs lus par `readlink`, cartes par leur chemin sans le suffixe « (deleted) »). Le coût mesuré est d'environ 12 ms (descripteurs) et 110 ms (avec les cartes) par élément. Le nom du logiciel vient de `/proc/<pid>/comm` (15 caractères au plus), jamais un chemin (SEC9).
- **Processus illisibles.** Ils sont **ignorés** : un fichier tenu par un autre compte ou par un processus protégé n'est pas vu. Cette limite est écrite et affichée en mode avancé. En revanche, si la lecture de `/proc` est **inutilisable** (absent, refus général : aucun descripteur lisible, pas même ceux de Breathly), la vérification est « impossible » et **chaque élément est refusé** avec la raison `open_check_unavailable` et une explication.
- **Windows, fichier.** On ouvre le fichier avec l'accès `DELETE` et `FILE_READ_ATTRIBUTES`, partage total, `FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS` (`OpenOptionsExt`, bibliothèque standard seulement). Succès : le fichier n'est pas tenu sans partage de suppression ; **ce handle reste ouvert pendant l'action** (en plus des dossiers verrouillés de l'ADR 0018), donc il n'y a pas d'écart entre la sonde et la suppression. Code 32 : fichier utilisé, élément ignoré (`in_use`). Toute autre erreur : élément ignoré par prudence (`open_check_failed`).
- **Partage de suppression (Windows).** Un logiciel qui a ouvert le fichier avec partage de suppression n'est **pas détecté** : Windows autorise sa suppression. La limite est documentée ; le Restart Manager pourra l'affiner (non essayé).
- **Dossiers.** Pour un élément 🟢 (suppression définitive), les fichiers ouverts sont **conservés** et le reste est supprimé : l'élément est rapporté « partiel » avec le nombre de fichiers conservés et le nom des logiciels (Linux : instantané ; Windows : le code 32 de chaque fichier, sans prévérification). Pour un élément 🟡 déplacé d'un bloc à la corbeille, **tout l'élément est refusé** (`in_use`) s'il contient un fichier ouvert.
- **Jamais de fermeture automatique.** Breathly ne ferme jamais un logiciel sans clic (invariant 10) ; l'explication dit quel logiciel tient le fichier et de le fermer avant de relancer l'analyse.
- **Résultat typé.** La vérification est un résultat typé (`libre`, `utilisé`, `impossible`), jamais une erreur du nettoyage ; les autres éléments continuent (US-036.1).

## Conséquences

- Ticket #73 (garde-fou « fichier ouvert ») : le trait, ses deux implémentations, le faux, les tests (un fichier réellement ouvert par un processus enfant dans un dossier temporaire sous Linux ; PowerShell ou un second handle sous Windows).
- Tickets #81 et #82 : l'état « partiel » d'un élément 🟢 et le refus d'un élément 🟡 ; le design 0001 §2 l'ajoute (un élément terminé garde le nombre de fichiers conservés).
- Ticket #80 (exécuteur) : la revérification du garde-fou « fichier ouvert » est faite pour chaque élément.
- L'essai de `trash::delete` sur un **dossier** contenant un fichier ouvert (boîte « Recycle Bin Access Denied » possible) reste un critère du ticket #81 (ADR 0019).
- `docs/policies/modele-de-menaces.md` : TS-01 couvre le garde-fou « fichier ouvert » sous les deux systèmes.

## Impact sur l'architecture

Un port de plus dans `breathly-platform` (design 0002 §5), sans changement du graphe de dépendances (design 0002 §3). Le design 0002 §9 (point sur la détection d'un fichier ouvert) passe à « décidé » dans la PR du ticket #73.
