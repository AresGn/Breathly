# Design 0003 : Écrans de l'application <mobile | web>

- **Statut** : base de travail pour les maquettes | maquettes livrées le AAAA-MM-JJ
- **Maquettes** : `maquettes/` (un fichier par écran) et son README
- **Sources** : PRD (stories, fonctions), design système (routes), ADR
- **Vocabulaire** : `GLOSSARY.md`

## 1. Objet et limites

Le PRD dit **quoi**, pas **quels écrans**. Ce document comble l'écart pour qu'un designer puisse produire les maquettes de toute l'application sans deviner. Il ne fixe ni l'identité visuelle, ni les textes définitifs, ni les composants. Décisions du propriétaire (datées) et ce qui est réservé au V2.

## 2. Plan de navigation

Arbre (ou diagramme Mermaid) : démarrage, états de compte, onglets.

## 3. Inventaire des écrans

Chaque écran porte un **identifiant stable** (A1, B3…) repris par la traçabilité, les états et les tickets.

| Id | Écran | Contenu et actions | Routes | Stories |
|---|---|---|---|---|

Règles de contenu par écran (ex. « ne jamais afficher X », « même erreur pour tout refus »).

## 4. Parcours à maquetter

Les chemins de bout en bout (première utilisation, rejet puis correction, hors ligne, notification qui ouvre un écran).

## 5. Traçabilité

Story → écran(s) ; route → écran(s).

## 6. États à maquetter

Pour **chaque écran** : chargement, erreur réseau avec « réessayer », liste vide. Puis les situations propres : hors ligne, compte en attente ou restreint, session expirée, limite de débit, permission système refusée, fichier invalide.

## 7. Textes à fournir

Les textes qui ne sont pas écrits ici et que le designer attend (engagement, messages d'erreur, notifications neutres).

## 8. Prévu en V2

Place à réserver, sans maquetter en détail.

## 9. Ce que ce document ne décide pas

Identité visuelle, palette (clair et sombre), typographie, composants.

## 10. Questions ouvertes
