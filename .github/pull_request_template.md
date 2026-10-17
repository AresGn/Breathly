## Ticket

<!-- `Closes #<n>` (ou `Part of #<n>` si le ticket est livré en plusieurs PR). Une PR = un ticket. -->

## Objet

<!-- Quoi et pourquoi, en 2-3 lignes. Lien vers l'ADR, le PRD ou le design si pertinent. -->

## Checklist architecture et sécurité

<!-- À adapter au projet (AGENTS.md, invariants métier). Garder une ligne par invariant non négociable. -->
- [ ] Aucun import ne traverse une frontière (`make arch` passe)
- [ ] Le Domain n'importe ni framework ni SDK tiers (si l'architecture en couches s'applique)
- [ ] Chaque invariant de `AGENTS.md` touché par cette PR est respecté et testé
- [ ] Aucune donnée personnelle (identifiant national, nom, téléphone, texte libre) dans les logs, notifications ou réponses publiques

## Checklist base de données

- [ ] Nouvelle table : clé primaire, horodatage de création avec fuseau, `updated_at` si mutable (conventions de `AGENTS.md`)
- [ ] Chaque clé étrangère a une action explicite ; tables de jonction avec clé primaire composite
- [ ] Contrainte métier critique verrouillée par la structure (unique, CHECK), pas seulement par le code

## Checklist clean code

- [ ] Chaque fonction fait une seule chose (test du « et / puis »)
- [ ] Aucun type flou (`any`, `mixed`, `dynamic`, `interface{}`, `object`) ni cast non justifié
- [ ] États en énumération ; `match` / `switch` exhaustif
- [ ] Aucune fonction > 40 lignes, aucun nesting > 3, aucun > 3 paramètres positionnels
- [ ] Aucune erreur avalée ; échecs métier attendus = résultat typé, pas exception
- [ ] Logique métier pure testable sans I/O
- [ ] Aucune abstraction pour un besoin qui n'existe pas encore (YAGNI)

## Sécurité par conception (`docs/policies/securite.md`)

- [ ] Route nouvelle : authentifiée, limitée en débit si sensible, test d'intrusion croisée (S1, S2, S5)
- [ ] Fichiers : stockage privé, URL signée courte (S7) ; journaux : identifiants seulement (S4)

## Contrat d'API

- [ ] Contrat régénéré (`make gen-api` ou équivalent) et commité, si la PR touche une route (sinon : « sans objet »)

## Documentation

- [ ] Documentation à jour dans cette PR : PRD ou design si une règle change, ADR si une décision change, `CODING.md` ou politique si une règle de code change, `GLOSSARY.md` si un terme est nouveau
- [ ] Sinon : « aucun impact documentaire » et pourquoi
