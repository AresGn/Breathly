# Détection d'un fichier ouvert : essais Linux et Windows, 2026-10-19

Alimente l'ADR 0020 (ticket #70 ; garde-fou S5, invariant 3 d'`AGENTS.md`, US-034.1, US-036.4).

## Question

Comment Breathly sait-il qu'un fichier est ouvert par un logiciel, sans lancer de programme externe (`lsof` est exclu, `CODING.md` §6), et avec quelle fiabilité ?

## Linux

Programme d'essai jetable (source : `fichiers-ouverts-2026-10-19/linux-proc.rs.txt`). Il ne lit que `/proc` et n'affiche que des comptes. Quatre processus enfants tiennent quatre fichiers d'essai : ouvert en lecture-écriture, ouvert en lecture seule, **cartographié en mémoire avec le descripteur fermé** (un petit enfant en Rust avec `mmap` de `libc`, puis fermeture), verrouillé par `flock`. Un cinquième fichier n'est jamais ouvert (témoin).

| Méthode | Lecture-écriture | Lecture seule | Cartographié, descripteur fermé | `flock` | Témoin |
|---|---|---|---|---|---|
| Descripteurs : `/proc/<pid>/fd` (identité dev et inode) | détecté | détecté | **non détecté** | détecté | non |
| Cartes mémoire : `/proc/<pid>/maps` (dev et inode) | non | non | détecté | non | non |
| `File::try_lock_shared` (bibliothèque standard) | `Ok` (non détecté) | `Ok` | `Ok` | `WouldBlock` (détecté) | `Ok` |

Mesures (machine de développement, 8 cœurs) : 342 processus ; **175 illisibles** (autre compte ou droits) ; un second relevé donne 174 illisibles dont **171 d'un autre compte et 3 du compte courant** ; environ 5 000 descripteurs lus. Instantané des descripteurs : **11 à 12 ms** ; avec les cartes : **104 à 115 ms** (3 séries) ; 1 000 consultations de l'instantané : 21 à 34 µs.

Un premier essai avait donné un faux « cartographié détecté par descripteur » : la bibliothèque `mmap` de Python **duplique** le descripteur et le garde ; l'essai a été refait avec `mmap` de `libc`.

Ce que la méthode ne voit pas : un fichier tenu par un processus que Breathly ne peut pas inspecter (autre compte, processus protégé) ; un fichier ouvert **après** l'instantané (course) ; un conteneur ou un bac à sable (snap, Flatpak) dont l'espace de noms de montage donne d'autres chemins : **non essayé**.

## Windows

Quatre appels de PowerShell tiennent un fichier d'essai ouvert avec un mode de partage différent ; le programme sonde ensuite par une ouverture avec accès `DELETE` (ce dont une suppression a besoin, partage total), une ouverture en lecture, `try_lock_shared` et `remove_file`. Poste client Windows 11 (build 10.0.26300), sans droits administrateur, programme `docs/research/essais-windows/` au commit `0b81cca` ; résultats bruts dans `fichiers-ouverts-2026-10-19/windows-resultats.txt`.

| L'autre processus tient le fichier... | Ouverture avec accès `DELETE` | Ouverture en lecture | `try_lock_shared` | `remove_file` |
|---|---|---|---|---|
| exclusivement (`FileShare.None`) | **erreur 32** | erreur 32 | erreur 32 | erreur 32 |
| lecture-écriture partagées, sans partage de suppression | **erreur 32** | OK | obtenu | erreur 32 |
| lecture-écriture partagées, **avec** partage de suppression | OK (**non détecté**) | OK | obtenu | OK (supprimé) |
| en lecture seule, lecture partagée | **erreur 32** | OK | obtenu | erreur 32 |

Lecture : l'échec de l'ouverture avec accès `DELETE` (code 32, violation de partage) **prédit exactement** l'échec de `remove_file` (4 cas sur 4). Le seul cas non détecté est un programme qui a ouvert le fichier avec partage de suppression ; Windows autorise alors la suppression. `try_lock_shared` ne voit pas un fichier ouvert (il n'échoue que par la violation de partage du mode exclusif).

## Non vérifié

- Windows 10, disques réseau ; le **Restart Manager** de Windows (liste des processus qui utilisent un fichier) : non essayé ; l'effet, sur le logiciel qui a ouvert le fichier avec partage de suppression, de la suppression de ce fichier.
- Linux : conteneurs, snap et Flatpak ; `hidepid` sur `/proc` ; une machine où `/proc` n'est pas monté ; le nom des processus (`/proc/<pid>/comm`, limité à 15 caractères) ; les dossiers (plusieurs milliers de fichiers ouverts sous un dossier) : l'essai a porté sur des fichiers.
- Le coût de 110 ms par instantané avec cartes multiplié par le nombre d'éléments d'un nettoyage : estimé (un élément est un résultat de règle, pas un fichier ; l'ordre de grandeur attendu est de quelques dizaines), non mesuré sur un vrai nettoyage.
