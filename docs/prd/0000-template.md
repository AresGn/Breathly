# PRD 0001 : <Nom du produit>

- **Statut** : brouillon | à relire | validé
- **Date** : AAAA-MM-JJ
- **Source** : cahier des charges, échanges, arbitrages datés
- **Lecteur** : équipe de développement (humains et agents)
- **Vocabulaire** : `GLOSSARY.md` fait foi.
- **Étiquettes** : chaque exigence est marquée **[V1]** ou **[V2]**. Hors étiquette = V1.

## Problem Statement

Le problème vu du point de vue de l'utilisateur, en 2 à 4 phrases, avec les conséquences concrètes (délai, erreur, danger, coût).

## Solution

La solution vue de l'utilisateur. Une phrase sur le **principe qui ne se négocie pas** (ex. « l'outil assiste, il ne décide pas »). Les grands composants (sans technologie : les choix techniques sont des ADR).

## Périmètre

| Domaine | V1 (MVP) | V2 |
|---|---|---|
| <domaine 1> | <ce qui est livré> | <ce qui est reporté> |

## Acteurs

| Acteur | Rôle | Étiquette |
|---|---|---|
| <acteur> | <ce qu'il fait> | V1 |

## User Stories

Une liste **longue et numérotée** (la numérotation est stable : les tickets la citent).

### <Groupe fonctionnel>

1. En tant que <acteur>, je veux <fonction>, afin de <bénéfice>.

## Exigences fonctionnelles

### F1. <Fonction> (source : cahier des charges §…)

- Règle 1.
- Règle 2.
- **Acceptation** : une phrase vérifiable par un test, avec le cas limite (ex. « un second compte pour le même identifiant est refusé par la même réponse que tout autre refus »).

### F2. <Fonction>

…

## Machines à états

États, transitions (de → vers), **permission requise** et **condition** de chaque transition. Un tableau par objet. Valider par un prototype jetable qui parcourt tous les états atteignables avant d'écrire les migrations (états sans sortie, états incohérents entre deux objets liés).

## Exigences non fonctionnelles

- **Authentification** : …
- **Autorisation** : permissions, pas rôles.
- **Audit** : journal append-only (qui, quoi, quand, sur quoi, ancienne et nouvelle valeur).
- **Protection des données** : …
- **Charge, disponibilité, sauvegardes** : …
- **Langue** : …
- **Conservation** *(valeurs proposées, à valider par la note de conformité)* : …
- **Hébergement** : …

## Matrice des permissions

| Permission | Effet |
|---|---|
| `action:ressource` | ce qu'elle autorise |

Les profils sont des conteneurs de ces permissions ; le client en fixe le contenu. Distinguer les permissions **sensibles** (attribuées par proposition et approbation de deux personnes distinctes, jamais contenues dans un profil).

## Implementation Decisions

Décisions de conception qui s'imposent au code (architecture, ports, séparation des vues, contrat d'API). Renvoyer vers les ADR.

## Testing Decisions

- **Un bon test** vérifie un comportement observable, jamais un détail d'implémentation.
- **Seam principal** : le plus haut point où l'on peut tester (souvent l'API en boîte noire contre une vraie base).
- **Tests obligatoires (défauts bloquants)** : une liste courte, une ligne par invariant.

## Critères de réussite

Reprise des critères du cahier des charges, réduits au périmètre de la V1 (un tableau « critère / V1 »).

## Out of Scope

- Tout ce qui est marqué [V2].
- <ce qu'on a décidé de ne pas faire, et pourquoi>

## Risques ouverts

1. <décision qui dépend du client ou d'un tiers>, ADR correspondant.

## Traçabilité

| Section du cahier des charges | Couverte par |
|---|---|
| 1 Présentation | Problème, Solution |
