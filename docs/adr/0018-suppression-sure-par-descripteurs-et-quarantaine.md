# ADR 0018 : Suppression sûre par descripteurs et quarantaine

- **Statut** : accepté ; Linux par la décision ci-dessous, Windows par la note du 2026-10-19 en fin de document
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Breathly vérifie les garde-fous S5 pour chaque élément au moment d'agir (SEC2, US-036.4), mais il reste un écart entre la vérification d'un chemin et l'action sur ce chemin : si un dossier du chemin devient un lien symbolique entre les deux, l'action atteint un fichier protégé. Le prototype P1 avait démontré la faille avec `trash::delete` (`docs/policies/securite.md` §3). Ticket #69 : comparer les contre-mesures, dire la fenêtre restante de chacune et ce que fait le cœur quand l'identité d'un dossier a changé.

**Mesuré (2026-10-18)**, sous Linux, dans un dossier temporaire avec la corbeille redirigée, détail et sources dans `docs/research/2026-10-18-suppression-sure.md` : un fil échange en boucle un dossier du chemin et un lien vers un dossier protégé, juste après la vérification (le pire moment possible, pas une probabilité réelle). Fichiers protégés perdus, deux séries de 2 000 essais :

| Variante | Pertes |
|---|---|
| vérifier puis `trash::delete(chemin)` | 1 032 et 1 039 |
| + recanonisation juste avant d'agir | 1 061 et 1 096 |
| + identité des dossiers avant et juste avant | 1 045 et 1 084 |
| détection après coup (origine enregistrée) puis restauration | 563 et 579 |
| suppression d'un fichier relative aux descripteurs (`openat` avec `O_NOFOLLOW`, `unlinkat`) | 0 et 0 |
| `renameat` dans une quarantaine privée du même volume, puis `trash::delete` | 0 et 0 |
| dossier : `remove_dir_all(chemin)` après vérification | 5 pertes sur 4 000 essais en 4 séries |
| dossier : suppression récursive relative aux descripteurs | 0 sur 2 000 |

**Non vérifié.** Windows (aucun essai) ; les dossiers 🟡 passés par la quarantaine ; la restauration quand l'emplacement d'origine est occupé ou son dossier disparu ; une quarantaine sur un autre volume ; les points de montage dans un dossier de cache.

## Options

1. **Descripteurs pour 🟢 et quarantaine pour 🟡 (retenue).**
2. Descripteurs pour 🟢 seulement ; `trash::delete(chemin)` direct pour 🟡 avec la fenêtre de la bibliothèque (mesurée large dans le pire cas) ; la détection après coup a laissé 28 % de pertes.
3. Vérifications de chemin seulement (recanonisation, identité des composants) : aucun effet sur le pire cas mesuré, suppression définitive irrécupérable.

## Décision

**Option 1.** Aucune action de suppression ne s'appuie sur un chemin revérifié : elle s'appuie sur des descripteurs ouverts sans suivre de lien.

- **Parcours.** Pour chaque élément, `breathly-platform` ouvre le dossier racine de la règle, puis chaque composant du chemin par `openat` avec `O_NOFOLLOW` et `O_DIRECTORY` (descripteur du dossier parent à chaque niveau). Un composant qui est devenu un lien ou qui a disparu fait **refuser** l'élément : état « ignoré » avec la raison `path_changed` (clé de traduction) et une explication (US-036.4). Ce refus est un résultat typé, pas une erreur du nettoyage : les autres éléments continuent.
- **Suppression définitive des caches 🟢** (S3, ADR 0005). Suppression récursive **relative aux descripteurs** : `fstatat` sans suivre de lien, `unlinkat` pour les fichiers et les liens, `AT_REMOVEDIR` après le contenu d'un dossier ; un lien est retiré, jamais traversé. La récursion ne franchit pas un autre volume (comparaison de `st_dev` avec celui de la racine de la règle).
- **Fichiers et dossiers de l'utilisateur 🟡** (corbeille). L'entrée atteinte par le parcours est d'abord déplacée par `renameat` (relatif aux descripteurs) dans un dossier de **quarantaine** privé de Breathly, **sur le même volume**, créé avec des droits limités à l'utilisateur (sous le dossier d'état de l'ADR 0003) ; `trash::delete` ne reçoit ensuite que le chemin de la quarantaine. Si le déplacement échoue (autre volume, `EXDEV`), l'élément est **ignoré avec une explication** : aucune suppression n'a lieu.
- **Historique** (ADR 0003 et son addendum). L'entrée « pending » est écrite avant l'action avec le **chemin d'origine** (`path`) ; la corbeille enregistre la quarantaine comme origine, ce qui est sans conséquence pour Breathly.
- **Annulation.** Avant de restaurer, Breathly vérifie que l'emplacement d'origine est libre et que son dossier existe (US-041.2) ; sinon l'annulation échoue avec une explication, sans rien déplacer. Puis `restore_all` remet l'élément dans la quarantaine (vérifié) et Breathly le déplace vers le chemin d'origine par le même parcours sans suivre de lien.
- **Reprise après un arrêt brutal** (US-040.6). Une entrée « pending » dont l'élément est retrouvé dans la quarantaine est remise à son origine si elle est libre ; sinon elle devient « incertaine » avec un message.
- **Code.** Ces opérations sont du code `unsafe` de `breathly-platform` (seul crate qui peut en avoir, ADR 0011), derrière un trait de suppression avec un faux de test, chacune avec son `#[allow(unsafe_code, reason)]`, son commentaire `// SAFETY:` et une relecture explicite. Aucun `cfg(target_os)` ailleurs.
- **Dépendance.** `libc` est accepté en dépendance directe de `breathly-platform` (MIT OR Apache-2.0, version 0.2.190 publiée le 2026-10-02, déjà présente dans l'arbre par `sysinfo`). Elle s'ajoute à la liste de l'ADR 0012 ; toute autre bibliothèque se redemande (PRD §14).
- **Windows.** Voir la note du 2026-10-19 en fin de document (essai fait ; stratégie par dossiers verrouillés).

## Conséquences

- Le ticket #72 (garde-fous et `VerifiedItem`) et le ticket #74 (liens et jonctions) s'appuient sur ce parcours ; le test TS-02 reproduit la course de cet essai (un fil qui échange un dossier et un lien) dans un dossier temporaire avec le faux `TrashProvider`.
- Les tickets #81 (corbeille), #82 (suppression définitive) et #71 (adapter `TrashProvider`) implémentent les deux chemins ; #85 (restauration) et le ticket de l'état des entrées (reprise après arrêt brutal) gèrent la quarantaine.
- `VerifiedItem` n'est plus un chemin revérifié : il porte l'identité du parcours (le descripteur du parent et le nom) ; une suppression n'accepte que lui.
- L'essai de course est conservé comme test de régression : la variante naïve doit perdre le fichier protégé, la variante retenue jamais.
- Le critère de la machine à états 0001 (revérification pour chaque élément) reste vrai : le parcours est refait pour chaque élément au moment d'agir.

## Impact sur l'architecture

Le trait de suppression est dans `breathly-platform` (design 0002 §5) ; `breathly-disk` n'a aucun appel système direct. Le graphe de dépendances (design 0002 §3) ne change pas. `docs/policies/securite.md` §3 (« Suppression sûre ») passe à « décidé » dans la PR du ticket #72.

## Note du 2026-10-19 : Windows

Essai fait sur Windows 11 (poste client, sans droits administrateur), détail et limites dans `docs/research/2026-10-19-essai-windows.md` (programme : `docs/research/essais-windows/`). Même méthode que sous Linux : un fil échange un dossier du chemin et une jonction vers un dossier protégé entre la vérification et l'action (trois renommages, faute d'échange atomique), avec une fenêtre de 10 ms ; « jonction vue » vérifie que l'échange a eu lieu.

| Variante | Perdus |
|---|---|
| fichier, vérifier puis `remove_file(chemin)` | 120 / 300 |
| dossier, vérifier puis `remove_dir_all(chemin)`, parent échangé | 57 / 200 |
| `trash::delete(chemin)` naïf | 64 / 100 |
| dossiers de la chaîne tenus ouverts sans partage de suppression, puis fichier / dossier / `trash::delete` | 0 / 300, 0 / 200, 0 / 100 |
| suppression d'un fichier par handle | 0 / 300 |

**Décision pour Windows (choisie par le propriétaire le 2026-10-19) : dossiers verrouillés, puis opération de la bibliothèque standard.** Aucune quarantaine, aucun `unsafe`, aucune dépendance de plus.

- Avant d'agir, `breathly-platform` ouvre la racine de la règle puis chaque dossier du chemin (`OpenOptions` en lecture, `share_mode` lecture et écriture **sans** partage de suppression, `FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT`) et **refuse** si l'un porte l'attribut `FILE_ATTRIBUTE_REPARSE_POINT` (raison `path_changed`). Tant que ces handles vivent, un autre processus ne peut ni renommer ni supprimer ces dossiers, donc ni les remplacer par une jonction (essai : erreur 32).
- Fichier (🟢 définitif ou 🟡 corbeille) : `remove_file` ou `trash::delete(chemin)` **pendant** que les handles vivent.
- Dossier 🟢 : le dernier dossier est aussi tenu ouvert ; son contenu est supprimé sous le verrou (une entrée point d'analyse est retirée par `remove_dir`, jamais suivie) ; puis le handle du dossier final est relâché et `remove_dir` retire le dossier (vide, ou une jonction : `remove_dir` ne détruit jamais la cible d'une jonction).
- Les handles sont relâchés dès la fin de l'élément. Un dossier impossible à ouvrir (utilisé par un autre programme, antivirus) donne un refus **sans perte** : élément ignoré avec explication (`path_busy`, US-036.4).
- **Un refus de la corbeille reste une erreur de l'élément**, jamais une suppression définitive (invariant 2) : voir l'ADR 0019.
- Le banc d'essai devient un **test de régression sous Windows** (CI `windows-latest`, ticket #29) : la variante naïve doit perdre le fichier protégé, la variante retenue jamais.

**Non vérifié (Windows).** Windows 10, disques réseau, ReFS, FAT ; liens symboliques de dossier ; un fichier ouvert par un autre programme pendant `trash::delete` (la boîte « Recycle Bin Access Denied » peut-elle apparaître malgré le verrouillage ?) ; la protection des ancêtres de la racine d'une règle, isolément ; un antivirus qui garde un dossier ouvert.
