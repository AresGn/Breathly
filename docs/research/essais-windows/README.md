# Essai Windows (tickets #49 et #69)

Programme jetable qui vérifie sous Windows ce que les ADR 0017 et 0018 n'ont pu vérifier que sous Linux : comportement des chemins (très longs, `\\?\`, casse, noms réservés, noms courts 8.3, jonctions, liens symboliques) et suppression sûre (jonction qui traverse le chemin, course entre vérification et action, suppression par handle, verrouillage de la chaîne de dossiers). Il **ne fait partie ni du produit ni de l'espace de travail Cargo** du dépôt (son `Cargo.toml` a sa propre section `[workspace]`).

## Ce qu'il touche

Seulement un dossier temporaire `%TEMP%\breathly-essai-<n>`, qu'il supprime à la fin. Aucun fichier de l'utilisateur n'est lu ni modifié. Avec l'option `--corbeille`, des fichiers d'essai (`breathly-essai-*`) passent par la vraie corbeille de Windows puis en sont retirés ; rien d'autre n'y est touché. Le rapport écrit (`rapport-essai-windows.txt`) masque le dossier temporaire et le nom d'utilisateur.

## Lancer l'essai

**Sur GitHub, sans rien installer** (Windows Server de GitHub, pas un poste client) : onglet *Actions* du dépôt, workflow `essai-windows`, *Run workflow* ; le rapport s'affiche dans le journal de l'étape « Rapport ».

**Sur un PC Windows** (poste client) : installer Rust (https://rustup.rs ; l'installation demande les outils de build C++ de Visual Studio) puis, dans un terminal :

```
git clone https://github.com/AresGn/Breathly.git
cd Breathly\docs\research\essais-windows
cargo run --release -- --essais 300 --delai-us 10000
```

Le programme affiche le rapport et l'écrit dans `rapport-essai-windows.txt` (dans ce dossier). Le coller dans la conversation, ou l'ajouter à `docs/research/` par une PR. `--essais N` règle le nombre d'essais de course par variante (300 par défaut, environ une minute) ; `--delai-us N` règle le délai, en microsecondes, que chaque variante laisse au fil perturbateur entre sa vérification et son action (10 000 par défaut, soit 10 ms ; une valeur plus petite resserre la fenêtre) ; sans `--corbeille`, la vraie corbeille n'est pas utilisée.

## Variantes de course comparées

1. **naïf** : `canonicalize` puis `remove_file(chemin)` ;
2. **par handle** : ouverture de l'objet sans suivre un point d'analyse final, chemin réel lu **par le handle**, comparaison à la racine autorisée, suppression **par le même handle** (`SetFileInformationByHandle`) ;
3. **chaîne verrouillée** : chaque dossier du chemin est tenu ouvert sans partage de suppression, de sorte qu'un autre processus ne peut pas le renommer.

Pour les dossiers (section B6), on compare `remove_dir_all` après vérification, les seuls parents verrouillés et les parents plus le dossier final verrouillés, en échangeant soit un dossier parent, soit le dernier composant. La section C2b ajoute `trash::delete` après verrouillage des dossiers. **Lecture de « vue »** : pour une variante qui verrouille, « vue 0 » est le résultat attendu, puisque le fil perturbateur ne parvient plus à renommer le dossier ; c'est la variante naïve, avec « vue » proche du nombre d'essais, qui montre que le banc déclenche bien l'échange.

Sous Windows il n'existe pas d'échange atomique de deux entrées : le fil perturbateur alterne trois renommages, ce qui crée des moments où le composant est la jonction, mais d'autres où il n'existe pas (refus). Chaque variante appelle un crochet entre sa vérification et son action : le banc y lance le fil et attend le délai, la fenêtre est donc la même pour toutes. Le rapport donne, par variante, le nombre d'essais où la jonction a été observée en place au moins une fois pendant la fenêtre (« vue ») : si ce nombre est nul, l'essai n'a rien prouvé.
