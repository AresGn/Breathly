# Règles de nettoyage

Chaque fichier `*.toml` de ce dossier décrit un ou plusieurs emplacements que Breathly peut proposer de nettoyer. **Aucune ligne de Rust à écrire** : déposez un fichier, ouvrez une PR, et la règle arrive dans le binaire à la version suivante (US-060.1). Breathly ne lit jamais ce dossier à l'exécution : les règles sont intégrées à la compilation (ADR 0004).

Format complet, chemins autorisés et refus : [`docs/design/0004-format-des-regles-de-nettoyage.md`](../docs/design/0004-format-des-regles-de-nettoyage.md).

## Ajouter une règle

1. Créez un fichier `rules/<outil>.toml` (minuscules, par exemple `pip.toml`), ou ajoutez une table `[[rule]]` à un fichier existant du même outil.
2. Partez de l'exemple ci-dessous (`rules/npm.toml`) :

   ```toml
   [[rule]]
   id            = "npm-cache"
   name.fr       = "Cache de npm"
   name.en       = "npm cache"
   explain.fr    = "Paquets téléchargés par npm. Ils sont retéléchargés automatiquement si besoin."
   explain.en    = "Packages downloaded by npm. They are re-downloaded automatically when needed."
   risk          = "safe"
   os            = ["linux", "windows"]
   kind          = "folder"
   paths.linux   = ["~/.npm/_cacache"]
   paths.windows = ["%LOCALAPPDATA%/npm-cache/_cacache"]
   command       = "npm cache clean --force"
   ```

3. Vérifiez la règle : `make check` (ou seulement `cargo test -p breathly-rules`). Une règle invalide fait échouer le test `every_rule_of_the_repository_is_valid` avec un message qui nomme le fichier, la règle et le défaut, par exemple :

   ```
   fichier de règles « pip.toml » : règle « pip-cache » : texte « explain.en » absent ou vide
   ```

4. Ouvrez la PR. Une règle nouvelle est relue par **deux personnes** (SEC6) ; une règle 🟢 qui supprime définitivement demande l'accord du propriétaire (PRD §14).

## À retenir

- **Explications en français et en anglais**, en langage simple : ce que c'est, et pourquoi on peut l'enlever sans risque (ou ce qu'il faut vérifier).
- **Niveau de risque** : `safe` 🟢 seulement pour un cache qui se recrée seul (forme `folder`) ; `review` 🟡 pour un dossier de projet, un installeur, la corbeille ; `personal` 🔴 n'est jamais sélectionnable.
- **Chemins** : séparateur `/`, racine `~` (Linux) ou `%LOCALAPPDATA%`, `%APPDATA%`, `%TEMP%`, `%USERPROFILE%` (Windows) ; jamais `Documents`, `Images`, `Vidéos`, `Musique`, `Bureau`, un dossier `.git` ni un profil de navigateur (S5). Un chemin par système listé dans `os`.
- **`command`** est facultatif et seulement affiché en mode avancé : Breathly ne l'exécute jamais (US-060.4).
- L'`id` est unique dans tout le dossier.
