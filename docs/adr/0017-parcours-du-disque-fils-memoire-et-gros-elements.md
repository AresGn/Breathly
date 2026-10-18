# ADR 0017 : Parcours du disque : fils, mémoire et gros éléments

- **Statut** : accepté
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

L'analyse du disque (US-021, critère S1 : environ 500 000 fichiers en moins de 60 s sur un SSD) tourne dans le processus de la fenêtre (design 0002 §4), dont le budget est de 350 Mo de PSS (ADR 0008, 288 Mo mesurés pour une page vide). Le design 0002 §8 veut que l'analyse « ne garde en mémoire que les totaux et les N plus gros éléments, pas les 500 000 chemins », avec un nombre de fils borné (hypothèse non vérifiée) ; le PRD ne donne ni N ni la définition d'un « gros dossier ». Ticket #48 (design 0002 §9 point 6).

**Mesuré (2026-10-18)**, sur le dossier personnel réel de la machine de développement (2,2 millions de fichiers, environ 4,5 fois la cible S1 ; NVMe, 8 cœurs ; lecture seule), méthode et résultats dans `docs/research/2026-10-18-parcours-du-disque.md` :

| Variante (cache chaud, médiane de 3 séries) | Durée | Mémoire maximale |
|---|---|---|
| Parcours standard, 1 fil | 8,3 s | 2 Mo |
| jwalk 0.9.0, 1 fil | 10,1 s | 11 Mo |
| jwalk, 2 fils | 5,8 s | 23 Mo |
| jwalk, 4 fils | 3,7 s | 39 Mo |
| jwalk, 8 fils | 3,0 s | 52 Mo |
| jwalk, 8 fils, tous les chemins gardés | 4,4 s | **714 Mo** |

Cache froid (première série, non contrôlée) : parcours standard 46 s, jwalk 4 fils 17 s, 8 fils 14 s, 2 fils 28 s. Toutes les variantes restent sous les 60 s de S1, froid compris.

**Non vérifié.** Disque dur mécanique, Windows, cache froid contrôlé ; la mémoire de la liste des gros dossiers (estimée à l'ordre du Mo) ; la PSS de l'application entière pendant une analyse (la somme « fenêtre + parcours » est une estimation) ; le coût de l'application des règles par-dessus le parcours seul.

## Options

**Fils.** 4 fils au plus (39 Mo ; environ 327 Mo avec la fenêtre) ; 8 fils (52 Mo ; environ 340 Mo, à 10 Mo du plafond) ; 2 fils (23 Mo, 5,8 s à chaud, 28 s à froid).

**Gros éléments.** 20 éléments et dossiers jusqu'à 3 niveaux (2 864 dossiers à suivre sous ce dossier personnel) ; 10 éléments jusqu'à 2 niveaux (563 dossiers) ; 50 éléments jusqu'à 4 niveaux (10 126 dossiers).

## Décision

- **Fils.** L'analyse du disque utilise **au plus 4 fils** : `min(4, nombre de cœurs)`, constante nommée dans le code, via le pool d'un `jwalk` dédié à l'analyse (pas le pool global). Raison : 4 fils tiennent 3,7 s à chaud pour 2,2 millions de fichiers, environ 17 s à froid, soit un facteur 3 à 4 sous S1 à 4,5 fois sa taille, pour environ 39 Mo de plus, sous le plafond de 350 Mo avec la fenêtre ; 8 fils ne gagnent qu'environ 20 % à chaud pour 13 Mo de plus.
- **Ce que l'analyse garde en mémoire.** Les **totaux par règle** (taille, nombre d'éléments, chemins des éléments de règle, qui sont peu nombreux), un **tas borné des N plus gros fichiers**, et le suivi des **dossiers jusqu'à 3 niveaux** sous le dossier personnel pour les gros dossiers. Elle ne garde **jamais** la liste des chemins de tous les fichiers (714 Mo mesurés pour 2,2 millions de fichiers, environ 320 octets par chemin).
- **Gros éléments (N).** Une liste unique de **20 éléments**, fichiers et dossiers mêlés, triés par taille (US-022). Un **gros dossier** est un dossier situé au plus à **3 niveaux** sous le dossier personnel, compté avec la **taille cumulée de son contenu**. Si un dossier et son parent seraient tous deux listés, **seul le parent** reste. Les éléments déjà couverts par une règle de nettoyage (caches 🟢 et 🟡) ne sont pas répétés dans cette liste. Ces valeurs sont des valeurs de départ, constantes nommées, réajustables par PR.
- **Contrôle.** Le ticket #57 (cible S1) mesure le parcours complet avec les règles et vérifie que la mémoire de l'analyse reste dans la borne (ordre de 40 à 50 Mo de plus que la fenêtre) ; un dépassement de 350 Mo de PSS mesurée est écrit sans arrondi et révise cet ADR ou l'ADR 0008 (jamais en silence).

## Conséquences

- Le ticket #56 (moteur d'analyse) applique la borne de 4 fils et les structures ci-dessus ; le ticket #58 (gros fichiers et dossiers) implémente la liste de 20 éléments sur 3 niveaux.
- Le critère S1 est tenu par la mesure sur la machine de référence ; Windows et disque mécanique restent à mesurer (ticket #131 pour la mémoire de la fenêtre, #132 et #133 pour les validations).
- Si un contributeur a un disque mécanique lent, le nombre de fils reste borné ; une valeur réglable viendra d'un ticket à part, pas de cet ADR.
- La décision n'ajoute aucune dépendance : `jwalk` est déjà accepté sous réserve de cette mesure par l'ADR 0012, et **cette mesure la confirme** (jwalk à 4 fils est environ 2,2 fois plus rapide que le parcours standard à chaud, 2,7 fois à froid).

## Impact sur l'architecture

Aucun changement du graphe de dépendances (design 0002 §3). Le design 0002 §8 (« hypothèse non vérifiée ») et §9 point 6 passent à « décidé » dans la PR du ticket #56.
