# ADR 0006 : Élévation des droits administrateur

- **Statut** : proposé
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Certaines actions exigent les droits administrateur (anciens noyaux, snap, flatpak, journaux systemd, caches apt et dnf, installation de zram ; V0.4). Le PRD (§7.8) pose : Breathly ne tourne jamais en administrateur, une action à la fois, après une explication, avec un mode guidé en repli, et ne manipule jamais un mot de passe. **La V0.1 n'a que le mode guidé** (commande à copier). Cet ADR prépare la V0.4.

Vérifié (page de manuel de pkexec, man.archlinux.org, lue le 2026-10-18) : pkexec exécute un programme avec les droits d'un autre compte après autorisation par polkit ; **il ne valide pas les arguments** qu'il passe au programme ; une action polkit peut être liée à un chemin de programme par l'annotation `org.freedesktop.policykit.exec.path`. **Non vérifié** : le mécanisme UAC côté Windows (manifeste `requireAdministrator` ou lancement avec élévation) et son comportement pour un programme signé ou non.

## Options

1. **Assistant à liste fermée** (retenu) : un binaire séparé `breathly-elevated`.
2. `pkexec` d'une commande par règle.
3. Mode guidé seulement.

## Décision

Option 1. Breathly (non élevé) lance `breathly-elevated` pour **une** action. Cet assistant n'accepte qu'un identifiant de règle de la liste fermée (E5) et revalide chaque paramètre contre les règles qu'il embarque lui-même ; il refuse toute commande libre. Sous Linux, une action polkit dédiée est liée au chemin de l'assistant ; sous Windows, l'assistant demande l'élévation à son lancement. L'application elle-même reste en droits utilisateur (E1). Le mot de passe est saisi dans la fenêtre de l'OS (E4).

## Conséquences

- Un composant de plus à construire, tester et signer (V1.0).
- Les règles d'élévation sont dans le binaire de l'assistant : une version modifiée de Breathly ne peut pas lui faire exécuter autre chose.
- À prouver par prototype avant la V0.4 : installation de l'action polkit (fichier `.policy` posé avec les droits qu'il faut), comportement de l'élévation sous Wayland, X11 et Windows 10/11, et message d'erreur quand l'utilisateur refuse (code de sortie 126 sous pkexec).
- Une distribution sans agent polkit ou un Windows sans droits : retomber sur le mode guidé (E3).
