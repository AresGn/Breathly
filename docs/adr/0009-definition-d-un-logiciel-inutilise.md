# ADR 0009 : Définition d'un logiciel inutilisé

- **Statut** : en attente
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

G3 et A1 demandent de signaler un logiciel gourmand **inutilisé** depuis 15 minutes (réglable). Aucun système ne fournit cette notion de façon uniforme (Wayland, X11, Windows). Sans définition précise, les fausses alertes ruineront la confiance.

## Hypothèse de travail

Un logiciel est inutilisé quand il n'a pas de fenêtre au premier plan, ne produit pas de son et garde un CPU faible pendant la durée choisie (PRD §6 V0.2). Breathly ne ferme jamais rien sans clic (invariant 10 d'`AGENTS.md`).

## À décider (après le prototype, étape 2.9)

Les signaux réellement accessibles par OS : fenêtre au premier plan (X11, Wayland, Windows), son en cours, enregistrement, appel, partage d'écran. Le prototype dit lesquels sont fiables ; l'ADR fixe la définition par OS et l'exclusion explicite de ce qui ne peut pas être détecté (par exemple, pas d'alerte du tout sous Wayland si aucun signal fiable n'existe).

## Conséquences attendues

US-120 et US-121 s'écrivent après cet ADR. Le module `app-watch` reste en V0.2 ; si la définition n'est pas fiable sur un OS, la fonction y est désactivée plutôt qu'imprécise.
