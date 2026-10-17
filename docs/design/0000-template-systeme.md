# Design 0001 : Système (modules, événements, contrat d'API)

- **Statut** : brouillon | révisé après relecture le AAAA-MM-JJ
- **Date** : AAAA-MM-JJ
- **Sources** : PRD, ADR, `AGENTS.md`
- **Vocabulaire** : `GLOSSARY.md`. Pour les termes de conception (module, interface, seam, adapter, profondeur), voir la skill `codebase-design`.
- **Étiquettes** : **[V1]** / **[V2]**.

Ce document décide *où vit chaque règle* et *par où on l'appelle*. Il ne décrit ni les tables ni le code.

## 1. Principes de conception

1. **Une règle, un seul endroit.** Chaque invariant est écrit et testé une fois.
2. **Interfaces étroites.** Un module expose quelques cas d'usage publics, rien d'autre. Test de suppression : si on supprime le module et que la complexité réapparaît chez N appelants, il est justifié.
3. **Un port seulement s'il varie.** Un port sans deuxième adapter est une indirection (sauf seam hypothétique assumé et documenté).
4. **Les modules se parlent par cas d'usage publics ou par événements de domaine**, jamais par les classes internes ni par des jointures sur les tables de l'autre.
5. **Pas d'exception à la règle de périmètre.**

## 2. Carte des modules

| Module | Possède | Interface publique (cas d'usage) | V1 |
|---|---|---|---|
| <module> | <données et règles> | <cas d'usage> | oui |

## 3. Dépendances autorisées

Une flèche signifie « appelle un cas d'usage public de ».

| Module | Dépend de |
|---|---|
| <module> | <modules> |

Le graphe d'appels est sans cycle, **le graphe d'événements aussi, et leur union aussi** : un événement n'est jamais émis dans le sens inverse d'un appel existant. Les tâches transverses (purge, désactivation d'un utilisateur, contrôle d'intégrité) sont des **orchestrateurs applicatifs hors graphe**.

## 4. Cloisonnement

La règle unique de visibilité (par affectation, service, propriétaire…), son mécanisme (projections locales mises à jour par événements), le comportement hors périmètre (404, pas 403) et la panne de la source (échec visible, jamais ressource invisible).

## 5. Lecture d'une donnée très sensible

Le cas d'usage unique, ses étapes (permission, périmètre, motif, audit écrit **avant** la donnée), ce qui n'est jamais renvoyé.

## 6. Vue interne et vue publique

Deux modèles, deux familles de ressources sans type commun, construction explicite champ par champ de la vue publique.

## 7. Événements de domaine

| Événement | Émis par | Consommé par |
|---|---|---|
| <Evenement> | <module> | <modules> |

Aucune charge utile ne contient de donnée sensible : des identifiants et un type seulement.

## 8. Groupes de routes

| Préfixe | Qui | Authentification | Contenu |
|---|---|---|---|
| `/auth/*` | tout le monde | aucune | … |
| `/me/*` | utilisateur final | jeton | … |
| `/internal/*` | profils internes | jeton + second facteur | … |

## 9. Catalogue des routes

| Méthode | Route | Rôle / permission |
|---|---|---|

## 10. Conventions du contrat

Erreur unique (enveloppe, codes), pas d'énumération (mêmes réponses pour tout refus d'inscription), pagination par curseur sur les listes qui reçoivent de nouveaux éléments, idempotence des créations (`Idempotency-Key`), nommage, évolution (champs optionnels seulement).

## 11. Décisions et points ouverts

Numérotés, chacun avec sa décision ou le ticket qui la porte.
