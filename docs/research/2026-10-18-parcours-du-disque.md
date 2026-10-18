# Parcours du disque : mesure de jwalk et du parcours standard, 2026-10-18

Alimente l'ADR 0017 (ticket #48 ; design 0002 §8 et §9 point 6 ; ADR 0008).

## Question

Combien de fils pour le parcours du disque, que garde l'analyse en mémoire (design 0002 §8 : « pas les 500 000 chemins »), et quelles bornes pour la liste des gros éléments ? Cible S1 / US-021.1 : environ 500 000 fichiers en moins de 60 s sur un SSD.

## Méthode

Banc d'essai jetable (hors dépôt ; source et résultats bruts dans `parcours-disque-2026-10-18/`), jwalk 0.9.0, Rust 1.97.1, build `release`. **Lecture seule** sur le dossier personnel réel de la machine de développement : le programme lit seulement les métadonnées (taille), ne lit aucun contenu, ne modifie rien et n'affiche aucun chemin. Pour chaque variante : nombre de fichiers et de dossiers, octets cumulés, durée totale, durée jusqu'au 500 000e fichier, mémoire maximale du processus (`VmHWM` de `/proc/self/status`, un RSS maximal, pas une PSS). Variantes : parcours standard (`read_dir` et `symlink_metadata`, un fil, pile explicite) ; jwalk avec 1, 2, 4 et 8 fils (`Parallelism::RayonNewPool`, la taille lue dans `process_read_dir`, liens non suivis, fichiers cachés inclus) ; jwalk 8 fils en gardant tous les chemins en mémoire. Chaque variante agrège les totaux et un tas des 20 plus gros fichiers.

**Machine** : 8 cœurs, 15,8 Go de RAM, NVMe ext4 plein à 96 %, noyau 6.1.0-53. **Arbre** : 2 227 000 fichiers et 314 886 dossiers, environ 132 Go (taille cumulée), soit environ 4,5 fois la cible S1 ; aucune erreur.

## Résultats

**Cache chaud** : trois séries consécutives après un échauffement ; machine peu chargée (charge moyenne autour de 3). Médiane des trois durées ; dispersion de moins de 5 %.

| Variante | Durée totale | 500 000 premiers fichiers | Mémoire maximale |
|---|---|---|---|
| Parcours standard, 1 fil | 8,3 s | 1,95 s | 2 Mo |
| jwalk, 1 fil | 10,1 s | 2,26 s | 11 Mo |
| jwalk, 2 fils | 5,8 s | 1,26 s | 23 Mo |
| jwalk, 4 fils | 3,7 s | 0,79 s | 39 Mo |
| jwalk, 8 fils | 3,0 s | 0,67 s | 52 Mo |
| jwalk, 8 fils, tous les chemins gardés | 4,4 s (une seule mesure) | 1,38 s | **714 Mo** (2 227 272 chemins) |

**Cache froid** : première série du jour, **non contrôlée** (cache de pages et de dentries peu rempli, machine chargée par d'autres tâches ; le cache n'a pas été vidé, ce qui exigerait les droits administrateur). À lire comme un ordre de grandeur, pas comme une mesure : standard 46 s, jwalk 1 fil 32 s, 2 fils 28 s, 4 fils 17 s, 8 fils 14 s ; la deuxième série, à moitié chaude, a donné 37 s, 35 s, 27 s, 16 s et 5,6 s.

## Ce que montrent les mesures

- Même à 4,5 fois la taille de la cible S1, **toutes les variantes parcourent l'arbre en moins de 60 s**, froid compris (le pire cas relevé : 46 s pour le parcours standard à un fil).
- jwalk à un fil n'est pas plus rapide que le parcours standard (10,1 s contre 8,3 s à chaud) ; son gain vient du parallélisme.
- Le gain décroît : 4 fils font 3,7 s, 8 fils 3,0 s (environ 20 % de mieux pour 13 Mo de plus). La mémoire croît d'environ 6 Mo par fil dans cette mesure.
- Garder tous les chemins coûte environ 320 octets par fichier (714 Mo pour 2,2 millions), contre 49 à 52 Mo en n'agrégeant que des totaux : c'est la raison de la règle « l'analyse ne garde pas les chemins ».
- La mesure est celle du parcours seul : sans application des règles, sans vérification du fichier marqueur, sans comparaison de chemins. Le critère S1 final se vérifie dans le ticket #57.

## Dossiers sous le dossier personnel (comptage `find`, sans noms)

| Profondeur | Dossiers |
|---|---|
| 1 | 92 |
| jusqu'à 2 | 563 |
| jusqu'à 3 | 2 864 |
| jusqu'à 4 | 10 126 |

## Non vérifié

- Disque dur mécanique, Windows, disque réseau, cache froid contrôlé.
- La mémoire de la liste des gros dossiers (2 864 entrées à suivre jusqu'à 3 niveaux, environ 300 octets chacune, soit de l'ordre du Mo : estimation, non mesurée).
- La PSS de l'application entière pendant une analyse : le total « fenêtre + parcours » est une addition d'une PSS (288 Mo, ADR 0008) et d'un RSS maximal, donc une estimation.
- L'effet du nombre de fils sur la réactivité de la fenêtre pendant l'analyse.
