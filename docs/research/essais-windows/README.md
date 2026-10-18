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
cargo run --release -- --essais 300 --corbeille
```

Le programme affiche le rapport et l'écrit dans `rapport-essai-windows.txt` (dans ce dossier). Le coller dans la conversation, ou l'ajouter à `docs/research/` par une PR. `--essais N` règle le nombre d'essais de course par variante (300 par défaut, environ une minute) ; sans `--corbeille`, la vraie corbeille n'est pas utilisée.

## Variantes de course comparées

1. **naïf** : `canonicalize` puis `remove_file(chemin)` ;
2. **par handle** : ouverture de l'objet sans suivre un point d'analyse final, chemin réel lu **par le handle**, comparaison à la racine autorisée, suppression **par le même handle** (`SetFileInformationByHandle`) ;
3. **chaîne verrouillée** : chaque dossier du chemin est tenu ouvert sans partage de suppression, de sorte qu'un autre processus ne peut pas le renommer.

Sous Windows il n'existe pas d'échange atomique de deux entrées : le fil perturbateur alterne trois renommages, ce qui crée des moments où le composant est la jonction, mais d'autres où il n'existe pas (refus).
