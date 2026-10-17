# Modèle de menaces

- **Statut** : à écrire **avant le premier sprint d'authentification**
- **Méthode** : STRIDE sur les 5 flux les plus critiques, une page chacun, pas un livre.
- **Question posée à chaque flux** : *« Que se passe-t-il si un attaquant décompile l'application, inspecte les requêtes avec un proxy, ou obtient une sauvegarde de la base ? »*

## Flux critiques

À REMPLIR: choisir 5 flux parmi : inscription / identité, lecture d'une donnée très sensible, publication vers un public, pièces jointes, notifications, paiement, export de données.

## Fiche d'un flux (à répéter)

### <Nom du flux>

- **Acteurs et frontières de confiance** : qui appelle quoi, ce qui est de confiance et ce qui ne l'est pas.
- **Actifs** : ce qu'on protège (donnée, droit, disponibilité).

| STRIDE | Menace | Parade (règle `S…` ou invariant) | Test qui la prouve |
|---|---|---|---|
| **S**poofing (usurpation) | | | |
| **T**ampering (altération) | | | |
| **R**epudiation (déni) | | audit append-only | |
| **I**nformation disclosure (fuite) | | | |
| **D**enial of service | | limite de débit | |
| **E**levation of privilege | | permissions, cloisonnement | |

## Sortie attendue

La liste des **tests de sécurité à écrire** au sprint d'authentification, une ligne par parade non encore couverte par un test.
