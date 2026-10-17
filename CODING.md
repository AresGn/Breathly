# . : écrire le code (Rust)

Règles d'écriture du code Rust de cette application. Elles complètent `AGENTS.md`, `docs/guides/principes-de-conception.md` et l'ADR d'architecture. **[machine]** casse `make check` ; **[relecture]** engage le relecteur.

> Pack **de départ, non éprouvé** : traduction en Rust des règles du projet de référence. **Vérifier chaque outil, option et version dans la documentation officielle à l'adoption**, puis marquer ce qui est vérifié.

Mot-guide : **le type porte la règle**. Un état impossible ne doit pas se pouvoir représenter.

## 1. Où va quoi

Un workspace Cargo : un crate par couche ou par module (`domain`, `application`, `infrastructure`, `api`). `domain` ne dépend d'aucun framework ni pilote ; les dépendances entre crates sont déclarées dans `Cargo.toml`, donc **vérifiées par le compilateur** **[machine]**. Visibilité minimale : `pub(crate)` par défaut.

## 2. Écrire une fonction

Seuils : fonction ≤ 40 lignes, imbrication ≤ 3 (`?` et retours anticipés), complexité ≤ 10 (`clippy::cognitive_complexity`), ≤ 3 paramètres (struct de paramètres au-delà), fichier ≤ 250 à 300 lignes. Un verbe une chose, nommage métier du `GLOSSARY.md`. Pas de booléen muet : `enum` à deux variantes.

## 3. Types

- Newtypes pour les identifiants et valeurs métier (`struct ItemId(Uuid)`) ; états en `enum` avec `match` **sans** branche `_` ; constructeurs qui valident (`TryFrom`) pour rendre les états invalides impossibles à construire.
- `unsafe_code = "forbid"` ; `unwrap()` interdit hors tests (`clippy::unwrap_used`) **[machine]** ; pas de `panic!` pour une erreur attendue.
- Immutabilité par défaut ; clonage justifié.

## 4. Erreurs

- `Result<T, E>` partout ; `E` est un `enum` d'erreurs métier typé (`thiserror`) pour une bibliothèque, `anyhow` seulement aux frontières d'un binaire. Un échec métier attendu est une variante, pas un `panic`.
- On ajoute du contexte avant de remonter ; jamais d'erreur ignorée (`let _ =` justifié par un commentaire).
- Les journaux (`tracing`) n'ont aucune donnée personnelle (liste blanche, JSON structuré, identifiant de requête).

## 5. Données et sécurité

- Authentification sur toute route sauf santé et routes publiques déclarées ; autorisation par permission ; cloisonnement dans le dépôt de données.
- Requêtes SQL paramétrées (vérifiées à la compilation si l'outil le permet) ; transactions explicites ; audit dans la même transaction.
- Secrets uniquement par variables d'environnement validées au démarrage ; `cargo audit` en CI **[machine]**.

## 6. Tests

- `cargo test` : tests unitaires du domaine sans I/O ; tests d'intégration en boîte noire contre une vraie base ; l'heure passe par un trait `Clock`.
- Par route : un test nominal et un test d'intrusion croisée. Aucun test ignoré (`#[ignore]`), supprimé ou affaibli pour passer au vert.

## 7. Code verbeux : ce qu'on retire

Commentaire qui répète le code, trait à une seule implémentation sans besoin de test, générique « au cas où » (YAGNI), règle métier écrite deux fois.

## 8. Définition de « terminé »

- [ ] `make ci-local` vert (`cargo fmt --check`, `clippy -D warnings`, `cargo test`, `cargo audit`)
- [ ] Aucun `unwrap()` hors tests, aucun `unsafe`
- [ ] Chaque échec métier attendu est une variante d'erreur
- [ ] Chaque route : authentifiée, permission vérifiée, test d'intrusion croisée
- [ ] Un test échoue sans le changement
