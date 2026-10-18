# ADR 0008 : Budgets de légèreté

- **Statut** : en attente
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

G5 : Breathly doit être léger. Le PRD §9 donne des cibles **non mesurées** : service Breathly < 15 Mo de RAM, interface ouverte < 150 Mo, installeur < 15 Mo, aucun processus quand la fenêtre est fermée (V0.1).

## Hypothèse de travail

Conserver ces cibles tant qu'aucune mesure ne les contredit.

## Mesures disponibles (prototype P2, Linux, 2026-10-18)

Un relevé complet des processus avec `sysinfo` prend environ **80 ms** ; un programme de mesure en version `release` pèse **744 Ko** et occupe environ **4,4 Mo** de RAM. La mesure de la fenêtre Tauri + Svelte (P4) n'est pas faite : les paquets de développement de WebKitGTK manquent (commande dans `docs/research/2026-10-18-prototypes-2-9.md`).

## À décider (après le prototype, étape 2.9)

Le prototype Tauri v2 + Svelte 5 mesure la RAM de la fenêtre (processus Breathly et moteur web du système), la taille de l'installeur `.msi`, `.deb`, `.rpm` et `.AppImage`, et le démarrage à froid sur les machines de validation. L'ADR fixe ensuite les budgets chiffrés par version, dit lesquels la CI contrôle (la taille de l'installeur, au minimum) et lesquels se contrôlent à la main.

## Conséquences attendues

US-072.2 devient vérifiable une fois cet ADR accepté. Si une cible est hors de portée, on la révise ici avec la mesure à l'appui, pas en silence.
