# extension : écrire le code (API Node.js / TypeScript)

Règles d'écriture d'une API Node.js en TypeScript (NestJS ou Express). Elles complètent `AGENTS.md`, `docs/guides/principes-de-conception.md` et l'ADR d'architecture. **[machine]** casse `make check` ; **[relecture]** engage le relecteur.

> Pack **de départ, non éprouvé** : écrit à partir des règles du projet de référence (API Laravel) traduites pour Node/TypeScript. **Vérifier chaque outil et chaque option dans sa documentation officielle à l'adoption** (skill `source-driven-development`), puis marquer ce qui est vérifié, comme le font les packs éprouvés.

Mot-guide : **frontière**. Tout ce qui entre (requête, message, variable d'environnement) est validé et typé à la frontière ; le cœur ne voit que des types précis.

## 1. Où va quoi

Quatre couches par module, comme dans les autres packs : `domain/` (règles, entités, ports), `application/` (un cas d'usage par classe ou fonction), `infrastructure/` (base de données, clients tiers, adapters), `http/` (contrôleurs ou routes, schémas de validation, sérialiseurs).

- `domain/` n'importe ni framework (NestJS, Express), ni ORM, ni SDK tiers **[machine : règle d'import ESLint ou test d'architecture]**.
- Un module n'importe pas les classes internes d'un autre : cas d'usage public ou événement.
- Un port (interface) se lie à son adapter dans la composition (module NestJS ou fonction de câblage), pas dans le domaine.

## 2. Écrire une fonction

Seuils : fonction ≤ 40 lignes, imbrication ≤ 3, ≤ 3 paramètres positionnels (objet nommé au-delà), fichier ≤ 250 à 300 lignes. Garde d'abord, un verbe une chose, nommage métier du `GLOSSARY.md`, booléens `is/has/can/should`.

## 3. Types et validation

- TypeScript `strict`, pas de `any` ni de `@ts-ignore` sans raison écrite **[machine]**.
- Chaque entrée est validée par un schéma (Zod, class-validator…) **avant** d'entrer dans `application/` ; les variables d'environnement aussi, au démarrage (échec rapide).
- Identifiants et valeurs métier en types brandés ; états en unions fermées avec `assertNever`.
- Les DTO de transport ne sont pas les entités du domaine ; la conversion se fait à la frontière.

## 4. Erreurs

- Échec métier attendu = valeur de retour typée (union discriminée ou `Result`), pas exception ; exception seulement pour un invariant violé.
- Un gestionnaire d'erreurs global traduit les exceptions en réponses d'erreur à enveloppe unique, sans détail interne.
- Jamais de `catch` vide ; une erreur avalée est un défaut. Les journaux ne contiennent aucune donnée personnelle (liste blanche, JSON structuré avec identifiant de requête).

## 5. Données et sécurité

- Authentification sur toute route sauf la santé et les routes publiques déclarées ; un test parcourt l'inventaire des routes.
- Autorisation par permission (`canUser`), jamais par nom de rôle ; cloisonnement appliqué dans le dépôt de données, jamais seulement dans le contrôleur.
- Limite de débit nommée sur les routes sensibles ; en-têtes de sécurité ; CORS restreint aux origines connues.
- Requêtes SQL paramétrées uniquement ; transactions explicites ; écriture d'audit dans la même transaction que la transition.
- Fichiers : stockage privé, type vérifié sur le contenu, nom généré, URL signée courte.
- Aucun secret dans le code ni dans l'image : variables d'environnement validées au démarrage.

## 6. Tests

- Vitest (ou Jest). Tests du domaine sans I/O ; tests d'API en boîte noire contre une vraie base ; l'heure passe par un port `Clock`.
- Pour chaque route : un test nominal et un test d'intrusion croisée (acteur hors périmètre).
- Aucun test sauté, supprimé, affaibli pour passer au vert.

## 7. Code verbeux : ce qu'on retire

Commentaire qui répète le code, couche de délégation sans valeur, interface à une seule implémentation sans besoin de test, abstraction « au cas où » (YAGNI), règle métier écrite deux fois.

## 8. Définition de « terminé »

- [ ] `make ci-local` vert
- [ ] Entrées validées à la frontière, aucun `any`
- [ ] Chaque échec métier attendu est une valeur de retour
- [ ] Chaque route : authentifiée, permission vérifiée, test d'intrusion croisée, limite de débit si sensible
- [ ] Aucune donnée personnelle dans les journaux
- [ ] Un test échoue sans le changement
