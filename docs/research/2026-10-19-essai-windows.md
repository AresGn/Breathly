# Essai Windows : chemins et suppression sûre, 2026-10-18 et 2026-10-19

Alimente l'ADR 0019 (ticket #49) et la note Windows de l'ADR 0018 (ticket #69). Complète `2026-10-18-suppression-sure.md` (Linux).

## Méthode

Programme jetable `docs/research/essais-windows/` (README : ce qu'il touche et comment le lancer), compilé avec Rust 1.97.1 sur le poste de développement Windows 11 (build 10.0.26300, **poste client, sans droits administrateur**), lancé par un agent sur ce poste ; les résultats ci-dessous sont ceux que l'agent a transmis, les rapports bruts (`rapport-essai-windows.txt`) ne sont pas dans le dépôt (ils sont masqués mais restent sur le poste). Le programme travaille dans `%TEMP%\breathly-essai-<n>`, qu'il supprime à la fin ; avec `--corbeille`, des fichiers `breathly-essai-*` passent par la vraie corbeille puis en sont retirés (169 éléments retirés au dernier passage).

Course : sous Windows il n'existe pas d'échange atomique de deux entrées ; un fil alterne en boucle trois renommages (`slot` dossier réel, `slot_alt` jonction vers un dossier protégé, `slot_tmp`). Chaque variante appelle un crochet **entre sa vérification et son action** ; le banc y lance le fil et attend `--delai-us` (10 ms par défaut). « Jonction vue » : essais où la jonction a été observée en place au moins une fois pendant la fenêtre. Les premiers passages ont montré que le banc était mal calibré (fenêtre trop étroite, compteur d'un seul instant) : ils sont décrits plus bas et corrigés par les commits `5048072`, `a6e1645` et `a94d702`.

## A. Chemins (résultats stables sur tous les passages)

| Cas | Résultat |
|---|---|
| Chemin de 347 caractères (nombreux niveaux de 40 caractères) | création, lecture des métadonnées, parcours (`read_dir`) et suppression récursive : réussis |
| `fs::canonicalize` | renvoie un chemin préfixé `\\?\` |
| `Path::starts_with` (chemin brut, racine canonique) | `false` ; avec les deux côtés canonicalisés : `true` |
| `Path::starts_with` entre majuscules et minuscules | `false` (sensible à la casse, alors que Windows ne l'est pas) ; `fs::canonicalize` normalise la casse |
| Noms `CON`, `NUL`, `aux.txt`, `COM1` | **créés** sans erreur (par le préfixe verbatim, qui contourne les noms réservés de Win32) |
| Nom court 8.3 | **indisponible** sur le disque d'essai (non vérifié) |
| Jonction (`mklink /J`, sans droits administrateur) | `is_symlink() = true` ; attribut `FILE_ATTRIBUTE_REPARSE_POINT` ; `canonicalize` la suit et sort de la zone autorisée (détectable) |
| Lien symbolique de dossier | **non testé** : `symlink_dir` échoue avec le code 1314 (privilège manquant, pas de mode développeur) |
| `remove_dir_all` d'un dossier qui contient une jonction | retire la jonction, la cible reste intacte |
| `remove_dir_all` et `trash::delete` sur un chemin qui **traverse** une jonction | atteignent le fichier protégé (B2, C1) |
| Dossier tenu ouvert sans partage de suppression | `rename` refusé (erreur 32) ; sans handle ouvert, accepté |

## B. Suppression sûre : course (dernière version du banc, commit `a94d702`)

Fichiers ou dossiers protégés perdus, essais par variante :

| Variante | Perdus | Jonction vue | Faits | Refus |
|---|---|---|---|---|
| B5 fichier, naïve (`canonicalize` puis `remove_file`) | 120 / 300 | 300 | 208 | 92 |
| B5 par handle (chemin réel lu et suppression par le même handle) | 0 / 300 | 0 | 300 | 0 |
| B5 chaîne de dossiers verrouillée | 0 / 300 | 0 | 300 | 0 |
| B6a dossier, parent échangé, naïve (`remove_dir_all`) | 57 / 200 | 199 | 139 | 61 |
| B6a parents verrouillés | 0 / 200 | 0 | 200 | 0 |
| B6a parents et dossier final verrouillés | 0 / 200 | 0 | 200 | 0 |
| B6b dernier composant échangé, naïve | 0 / 200 | 200 | 135 | 65 |
| B6b parents verrouillés seulement | 0 / 200 | 199 | 140 | 60 |
| B6b parents et dossier final verrouillés | 0 / 200 | 0 | 200 | 0 |

Corbeille de Windows (option `--corbeille`, 100 essais) :

| Variante | Perdus | Jonction vue | Faits | Refus | Durée |
|---|---|---|---|---|---|
| C2 `trash::delete` naïf | 64 / 100 | 100 | 69 | 31 | 39,7 s |
| C2b dossiers verrouillés puis `trash::delete` | 0 / 100 | 0 | 100 | 0 | 9,4 s |

Passages précédents (même poste), pour mémoire : fenêtre de 300 µs : 0 perdu pour les trois variantes (« vue 0 » partout, banc trop étroit), `trash::delete` naïf 35 / 200 perdus ; fenêtre de 2 000 µs : naïve 1 / 300 perdu ; fenêtre de 10 ms : naïve 42 / 300 (vue 294), par handle et chaîne verrouillée 0 / 300 ; avec corbeille : naïve 53 / 200 (vue 191), `trash::delete` naïf 33 / 200 (vue 194).

## Lecture

- **Pour une variante qui verrouille, « jonction vue 0 » est le bon résultat** : le fil perturbateur ne parvient plus à renommer le dossier. C'est la variante naïve (vue proche du nombre d'essais) qui prouve que le banc déclenche l'échange.
- Les variantes « par handle » et « chaîne verrouillée » n'ont jamais vu la jonction (« vue 0 », aucun refus) : c'est cohérent avec un handle ouvert qui empêche le renommage du dossier. Seul l'essai B4 l'a vérifié **directement** (dossier tenu ouvert sans partage de suppression : `rename` refusé, erreur 32) ; pour la variante « par handle » (handle sur le fichier, partage de suppression autorisé), l'empêchement du renommage est une inférence, pas une observation isolée.
- `remove_dir_all` de la bibliothèque standard n'a jamais suivi une jonction posée comme **dernier** composant (0 perte sur 200) ; la fenêtre est dans les composants **parents**.
- Parents seuls verrouillés : le dernier composant reste échangeable (60 refus avec jonction vue 199), sans perte avec `remove_dir_all`.
- Verrouiller coûte moins que le refus : C2b 9,4 s contre 39,7 s pour C2.

## Boîte de dialogue de la corbeille

Pendant les passages avec `--corbeille`, Windows a affiché « **Recycle Bin Access Denied** : `breathly-essai-cible.bin` can't be moved to the Recycle Bin. Do you want to permanently delete it? » avec les boutons *Yes* et *No*, pour un fichier d'essai de 8 à 12 octets. Elle est apparue plusieurs fois, jamais sans `--corbeille`, et revient à chaque refus de la corbeille ; la réponse *Yes* l'efface définitivement. Dans `trash` 5.2.9 (`windows.rs`, code lu), `trash::delete` passe `FOF_NO_UI | FOF_ALLOWUNDO | FOF_WANTNUKEWARNING` : la bibliothèque demande une opération sans interface et d'être avertie avant une destruction définitive, et la boîte apparaît malgré tout. Avec les dossiers verrouillés (C2b), aucun refus n'a eu lieu (0 sur 100) ; le nombre exact d'apparitions de la boîte au dernier passage n'est pas connu.

## Non vérifié

- Windows 10, disques réseau, ReFS, volumes sans verrouillage de partage (FAT) ; un poste avec noms courts 8.3 activés ; les liens symboliques de dossier (code 1314) ; `jwalk` sur des chemins longs ou à travers des jonctions ; la suppression d'un élément de nom réservé ; les noms finissant par un point ou une espace (le rapport transmis ne les détaille pas) ; la casse Unicode au-delà de l'ASCII.
- Un fichier ouvert par un autre programme pendant `trash::delete` : la boîte de dialogue peut-elle apparaître malgré le verrouillage ? Les dossiers verrouillés l'ont évitée dans l'essai, qui ne tient aucun fichier ouvert.
- Les ancêtres de la racine d'une règle : le handle ouvert sur la racine les protège-t-il aussi du renommage ? (Observé pour un parent direct, pas isolé pour les niveaux au-dessus.)
- Un antivirus qui garde un dossier ouvert : l'ouverture du verrou peut échouer, ce qui donne un refus (sans perte).
