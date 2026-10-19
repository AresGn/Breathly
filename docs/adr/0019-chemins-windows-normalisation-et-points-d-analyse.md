# ADR 0019 : Chemins Windows : normalisation et points d'analyse

- **Statut** : accepté (essais faits sur Windows 11, poste client ; Windows 10 et autres volumes non vérifiés)
- **Date** : 2026-10-19
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Les garde-fous (S5) et la résolution des règles comparent des chemins ; sous Windows, les chemins ont des particularités (préfixe `\\?\`, casse indifférente, chemins de plus de 260 caractères, noms réservés, noms courts 8.3, jonctions et liens symboliques). Le design 0002 §9 point 7 et le ticket #49 demandaient les règles de normalisation à appliquer avant tout contrôle de garde-fou, et la liste des cas à tester (US-034.3, TS-02). Les essais Windows de `docs/research/essais-windows/` ont été faits les 2026-10-18 et 2026-10-19 ; résultats et limites dans `docs/research/2026-10-19-essai-windows.md`.

**Vérifié (Windows 11, poste client, sans droits administrateur).**
- Un chemin de 347 caractères se crée, se lit, se parcourt (`read_dir`) et se supprime avec la bibliothèque standard.
- `fs::canonicalize` renvoie un chemin préfixé `\\?\` ; `Path::starts_with(chemin brut, racine canonique)` est faux ; avec les deux côtés canonicalisés, il est vrai.
- `Path::starts_with` est sensible à la casse (faux entre majuscules et minuscules) alors que Windows ne l'est pas ; `canonicalize` normalise la casse.
- Les noms `CON`, `NUL`, `aux.txt`, `COM1` ont pu être créés (par le préfixe verbatim).
- Une jonction (`mklink /J`, sans droits administrateur) est vue comme `is_symlink() = true` et porte l'attribut `FILE_ATTRIBUTE_REPARSE_POINT` ; `canonicalize` la suit et sort de la zone autorisée.
- `remove_dir_all` d'un dossier qui contient une jonction retire la jonction et laisse la cible intacte ; `remove_dir_all` et `trash::delete` sur un chemin qui traverse une jonction atteignent le fichier protégé.

**Non vérifié.** Windows 10, disques réseau, ReFS, FAT ; les noms courts 8.3 (indisponibles sur le disque d'essai) ; les liens symboliques de dossier (privilège manquant, code 1314) ; `jwalk` sur des chemins longs et à travers des jonctions ; la suppression d'un élément de nom réservé ; les noms finissant par un point ou une espace ; la casse Unicode au-delà de l'ASCII.

## Options

**Comparaison de chemins.** Canonicaliser des deux côtés et comparer sans tenir compte de la casse ; ou comparer des chaînes brutes ; ou ajouter une bibliothèque de normalisation. **Points d'analyse dans un arbre de cache.** Retirer le lien sans le suivre ; ou ignorer tout dossier qui en contient un.

## Décision

- **Un seul endroit.** `breathly-platform` fournit la fonction qui dit si un chemin est dans une racine ; aucun autre crate ne compare des chemins à la main.
- **Canonicaliser des deux côtés.** Le chemin à tester et la racine sont passés par `fs::canonicalize` avant comparaison ; on ne compare jamais un chemin brut à un chemin canonique. Une erreur de canonicalisation refuse l'élément.
- **Casse.** Sous Windows, la comparaison se fait composante par composante **sans tenir compte de la casse** (le programme d'essai la comparait sur les chaînes en minuscules). Limite : la casse des caractères non ASCII n'est pas vérifiée.
- **Préfixe `\\?\`.** Les chemins canoniques portent le préfixe ; il est retiré **à l'affichage seulement** (écrans, historique lisible), jamais avant une comparaison ni une opération. Le préfixe permet aussi les chemins longs : on travaille toujours sur des chemins absolus et la bibliothèque standard fait le reste.
- **Chemins longs.** Un chemin de plus de 260 caractères n'est pas un cas particulier du cœur ; le test de la validation parcourt un arbre de plus de 300 caractères avec le moteur d'analyse (`jwalk`, non essayé sous Windows).
- **Points d'analyse (jonctions, liens symboliques, autres).** Détectés par `symlink_metadata` (`FileType::is_symlink`, vrai pour une jonction, et l'attribut `FILE_ATTRIBUTE_REPARSE_POINT`) : un **composant** du chemin qui en est un fait **refuser** l'élément (raison `path_changed`, ADR 0018). **À l'intérieur** d'un dossier à nettoyer, un lien est **retiré sans être suivi** (comportement de la bibliothèque standard vérifié), jamais traversé ; l'analyse du disque ne descend pas dans un lien (`follow_links` faux) et ne compte pas son contenu.
- **Noms réservés, noms courts 8.3, liens symboliques de dossier.** Pas de traitement spécial dans le cœur ; leur essai est un critère des tickets de validation (voir « Conséquences »). Breathly ne crée jamais un nom réservé.
- **Suppression.** Elle suit la note Windows de l'ADR 0018 : dossiers de la chaîne tenus ouverts sans partage de suppression, vérification qu'aucun n'est un point d'analyse, puis l'opération. Aucune comparaison de chemin n'est une protection à elle seule.
- **Corbeille de Windows : un refus reste un échec.** `trash::delete` passe `FOF_NO_UI | FOF_ALLOWUNDO | FOF_WANTNUKEWARNING` (code de `trash` 5.2.9 lu), et malgré cela Windows a affiché « Recycle Bin Access Denied : Do you want to permanently delete it? » quand la corbeille refusait un fichier. Breathly ne répond jamais à cette boîte et ne propose jamais cette suppression : un refus de la corbeille est une **erreur de l'élément** avec explication (invariant 2 d'`AGENTS.md`), et le fichier reste en place.

## Conséquences

- Ticket #72 (garde-fous) et #74 (liens et jonctions) : la fonction de comparaison unique, le refus d'un composant point d'analyse et le test TS-02 sous Windows (le banc d'essai devient un test de régression : la variante naïve perd le fichier protégé, la variante retenue jamais) ; ils s'appuient sur la CI sous `windows-latest` (#29).
- Ticket #56 et #57 (moteur d'analyse) : `follow_links` faux, test sous Windows avec un chemin de plus de 300 caractères et une jonction dans l'arbre.
- Ticket #81 (corbeille) : un refus de `trash::delete` est une erreur de l'élément ; un essai avec un fichier **ouvert par un autre programme** vérifie que la boîte « Recycle Bin Access Denied » n'apparaît pas, ou ne bloque pas la tâche (la boîte est modale : l'annulation coopérative de l'ADR 0013 ne peut pas l'interrompre).
- Tickets de validation (#133, #134) : poste Windows 10, noms courts 8.3 activés, liens symboliques de dossier avec le mode développeur, noms réservés et noms finissant par un point.
- `docs/policies/modele-de-menaces.md` : TS-02 couvre les jonctions et les liens symboliques sous Windows.

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Le design 0002 §9 point 7 (« chemins Windows très longs et jonctions dans les garde-fous ») passe à « décidé » dans la PR du ticket #72.
