# Suppression sûre : course entre la vérification et l'action, essai du 2026-10-18

Alimente l'ADR 0018 (ticket #69 ; sécurité §3 ; SEC2 ; prototype P1).

## Question

Le prototype P1 a démontré une faille : si un dossier du chemin devient un lien symbolique entre la vérification (garde-fous S5) et l'action, `trash::delete` suit le lien et envoie à la corbeille un fichier protégé. Quelles contre-mesures la ferment, lesquelles ne font que la rétrécir, et à quel coût ?

## Méthode

Programme d'essai jetable, Linux (noyau 6.1.0-53, 8 cœurs, ext4), Rust 1.97.1, `trash` 5.2.9, `libc` 0.2.190 ; sources et résultats bruts dans `suppression-sure-2026-10-18/`. **Tout se passe dans un dossier temporaire** ; la corbeille est redirigée par `XDG_DATA_HOME` vers ce dossier et `HOME` vers un dossier vide ; la vraie corbeille de la machine contenait 0 élément avant et après chaque série.

Arborescence : `allowed/slot/target.bin` (le fichier à nettoyer), `allowed/slot_alt` (un lien vers `protected/`), `protected/target.bin` (le fichier à ne jamais toucher). Un fil échange en boucle, de façon atomique (`renameat2` avec `RENAME_EXCHANGE`), `slot` et `slot_alt`, donc `slot` est tantôt le vrai dossier, tantôt un lien vers `protected/`. À chaque essai, le programme vérifie le chemin (`canonicalize` dans `allowed/`), lance le fil d'échange, appelle la variante de suppression, puis regarde si `protected/target.bin` existe encore. Un essai de dossier (`d0`, `d6`) fait de même avec `allowed/slot/cachedir` (5 fichiers et un sous-dossier) et `protected/cachedir/keep.bin`.

**Limite de la méthode** : le fil d'échange démarre juste après la vérification. Les taux mesurent donc le **pire moment possible**, pas une probabilité réelle. Les taux de pertes des variantes qui ne protègent pas sont élevés parce que la fenêtre de la bibliothèque `trash` est large (plusieurs appels système sur le chemin avant le déplacement) ; ils ne disent pas qu'un utilisateur perdrait un fichier une fois sur deux.

## Variantes

| Variante | Principe |
|---|---|
| v0 | vérifier (`canonicalize` dans `allowed/`), puis `trash::delete(chemin)` |
| v1 | v0 avec une seconde vérification juste avant d'agir |
| v2 | refus si un composant est un lien (`lstat` de chaque niveau), identité (dev, ino) de chaque niveau relevée avant et juste avant, puis `trash::delete(chemin)` |
| v3c | v0, puis après l'action on compare l'origine enregistrée par la corbeille au chemin vérifié ; si elle diffère, `restore_all` du nouvel élément |
| v4 | suppression définitive d'un **fichier** : ouverture de `allowed/` puis de chaque niveau avec `openat` (`O_NOFOLLOW`, `O_DIRECTORY`), `fstatat` sans suivre de lien, `unlinkat` |
| v5 | l'entrée vérifiée est déplacée par `renameat` (relatif aux descripteurs, sans suivre de lien) dans un dossier de quarantaine privé du même volume, puis `trash::delete` depuis la quarantaine |
| d0 | suppression définitive d'un **dossier** : vérifier, puis `std::fs::remove_dir_all(chemin)` |
| d6 | suppression définitive d'un dossier, entièrement relative aux descripteurs (`openat`, `fdopendir`, `unlinkat`, `AT_REMOVEDIR`, récursion sans suivre de lien) |

Deux premières versions de v3 (comparaison d'inode, puis recherche du plus récent élément) ont donné des résultats faux (nom de fichier de la corbeille mal déduit, secondes de `time_deleted` ambiguës) ; elles ont été corrigées en v3c, qui compare l'origine enregistrée.

## Résultats

Chaque ligne : fichiers protégés perdus sur le nombre d'essais, deux séries.

| Variante | Série 1 | Série 2 | Durée par essai |
|---|---|---|---|
| v0 naïf | 1 032 / 2 000 | 1 039 / 2 000 | 0,9 ms |
| v1 recanonisation | 1 061 / 2 000 | 1 096 / 2 000 | 0,9 ms |
| v2 identité des composants | 1 045 / 2 000 | 1 084 / 2 000 | 0,9 ms |
| v3c détection après coup | 563 / 2 000 | 579 / 2 000 | 2,9 ms |
| v4 fichier, descripteurs | 0 / 2 000 | 0 / 2 000 | 0,28 ms |
| v5 quarantaine puis corbeille | 0 / 2 000 | 0 / 2 000 (et deux séries de mise au point : 0 / 4 000) | 0,66 ms |
| d0 dossier, `remove_dir_all` | 3 / 1 000 | 1 / 1 000 (deux séries plus anciennes : 0 et 1 sur 1 000) | 0,65 ms ; 59 à 64 % d'erreurs de suppression |
| d6 dossier, descripteurs | 0 / 1 000 | 0 / 1 000 | 0,65 ms |

## Ce que montrent les mesures

- **Re-vérifier le chemin ne protège pas** : v1 et v2 ont le même taux que le code naïf. La fenêtre qui reste est dans l'appel de la bibliothèque, après toute vérification possible côté Breathly.
- **Détecter après coup n'est pas fiable** : v3c laisse encore 28 % de pertes. Raison relevée par essai séparé : la corbeille enregistre l'origine comme le **chemin résolu** (`protected/target.bin` quand le chemin passe par un lien), et la restauration y retourne, mais la bibliothèque calcule ce chemin avant le déplacement ; l'origine enregistrée peut donc être fausse par rapport à ce qui a été déplacé.
- **Les opérations relatives aux descripteurs ferment la fenêtre** : 0 perte sur 2 000 essais de chaque série pour les fichiers (v4), les dossiers (d6) et la quarantaine (v5), pour un coût par essai égal ou inférieur au code naïf.
- `std::fs::remove_dir_all` est presque sûr (ouverture du dossier cible sans suivre de lien d'après la bibliothèque standard) mais une perte a été relevée dans plusieurs séries (5 sur 4 000 essais en tout) ; elle n'est pas fermée.
- **Restauration après quarantaine** (essai séparé, `restauration-quarantaine.rs.txt`) : la corbeille enregistre la quarantaine comme origine ; `restore_all` remet le fichier dans la quarantaine ; un `rename` vers le chemin d'origine conservé par Breathly (historique) le rend à sa place, contenu intact.

## Non vérifié

- **Windows** : aucun essai. Les mêmes principes existent (ouverture relative à un handle de dossier, `FILE_FLAG_OPEN_REPARSE_POINT`), mais ni leur emploi avec `trash` (qui passe par un appel orienté chemin) ni les jonctions ni les chemins longs n'ont été essayés (ticket #49).
- Les dossiers envoyés à la corbeille par quarantaine (l'essai a porté sur des fichiers) ; la restauration quand l'emplacement d'origine est occupé ou son dossier disparu.
- La quarantaine sur un autre volume : `renameat` échoue (`EXDEV`) ; l'essai n'a utilisé qu'un volume.
- Les points de montage dans un dossier de cache (comparaison de `st_dev`).
- La fermeture complète de la fenêtre d'un attaquant qui connaît le nom de la quarantaine (même utilisateur).
- Les modes de verrouillage et les fichiers ouverts (ticket #70).
