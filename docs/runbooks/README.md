# Runbooks d'exploitation

Un runbook décrit pas à pas une opération critique de production : rotation d'un certificat ou d'une clé, révocation en masse des jetons, sauvegarde et restauration, bascule.

**Règle : à la demande, avec le premier besoin réel.** Un runbook écrit avant l'opération est faux dès le déploiement. Déclencheurs habituels : l'hébergement est choisi, le modèle de jetons est décidé, une clé de chiffrement existe.

Gabarit : objectif, prérequis, étapes numérotées avec la commande exacte, vérification de la réussite, retour arrière, qui prévenir.
