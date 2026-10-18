# ADR 0008 : Budgets de légèreté

- **Statut** : accepté (valeurs de départ, re-mesurées à chaque version)
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

G5 : Breathly doit être léger, et l'architecture est pensée dès le départ pour le rester. Le PRD §9 donnait des cibles **non mesurées** : service Breathly < 15 Mo de RAM, interface ouverte < 150 Mo, installeur < 15 Mo, aucun processus fenêtre fermée. Les prototypes de l'étape 2.9 les ont confrontées à la mesure.

## Mesures (Linux, 2026-10-18)

Détail et méthode : `docs/research/2026-10-18-prototypes-2-9.md` (P2) et `docs/research/2026-10-18-coquille-desktop.md` (P4, P6).

| Objet | Mémoire (PSS) | Processus | Taille |
|---|---|---|---|
| Fenêtre Tauri + Svelte, une page vide (`release`) | **288 Mo** (RSS 461 Mo) | 3 (application et deux processus de WebKitGTK) | binaire 10,7 Mo ; `.deb` et `.rpm` 2,78 Mio ; `.AppImage` 98,4 Mio |
| Fenêtre native egui, une page vide (comparaison jetable, P6) | 96 Mo (RSS 106 Mo) | 1 | binaire 27,3 Mo |
| Programme de mesure `sysinfo` (annonce du coût du futur service) | RSS 4,4 Mo | 1 | binaire 0,74 Mo |

Un relevé complet des processus coûte environ 80 ms. La coquille Tauri s'ouvre en 0,28 s et ne laisse aucun processus après sa fermeture. **Windows (WebView2) n'est pas mesuré.**

## Décision

1. **L'interface reste Tauri v2 + Svelte 5 pour la V0.1.** Le moteur web du système coûte environ 190 Mo de plus qu'une fenêtre native minimale, mais les maquettes sont en HTML/CSS, l'accessibilité et l'internationalisation passent par des outils éprouvés, et G6 (contributeurs web) est tenu. L'interface ne parle au cœur que par des commandes (design 0002 §6) : elle reste remplaçable sans toucher aux crates. La comparaison native (egui) n'a mesuré qu'une page vide : fidélité à la maquette, accessibilité et effort d'une interface native ne sont **pas vérifiés**.
2. **Budget de la fenêtre : 350 Mo de PSS au plus**, sur les machines de validation (Linux et Windows). C'est la valeur mesurée (288 Mo) plus environ 20 % pour de vrais écrans, pas une promesse de légèreté : elle se **re-mesure à chaque version** et cet ADR la révise avec la mesure à l'appui, jamais en silence. Le contrôle est manuel (la CI n'a pas d'affichage). La PSS répartit la mémoire partagée entre les processus ; sous Windows, la mesure équivalente est à choisir à la première mesure.
3. **Budget du service Breathly (V0.2) : 15 Mo de RAM, 0,5 % de CPU** (PRD M1), inchangé : il est un binaire Rust séparé, sans moteur web ; le programme de mesure de 4,4 Mo montre que c'est atteignable, pas que c'est acquis.
4. **Installeurs** : `.deb` et `.rpm` < 15 Mo (mesurés à 2,78 Mio) ; `.msi` à mesurer lors de la première construction sous Windows. **Pas d'AppImage en V0.1** : elle embarque WebKitGTK (98,4 Mio) et sa construction télécharge des outils sans empreinte ; elle reviendra avec ses outils épinglés par empreinte (SEC11).
5. **Aucun processus de l'interface quand la fenêtre est fermée** : tenu par la coquille, contrôlé par US-072.1.

## Conséquences

- Le PRD §9 (Légèreté), le §6 (livrables de la V0.1) et les stories US-072.2 et US-073.1 sont alignés sur cette décision.
- `tauri.conf.json` ne construit plus que `deb`, `rpm` et `msi`.
- Le contrôle de CI de l'installeur (US-072.2) vérifie la taille du `.deb` et du `.rpm` ; la mémoire de la fenêtre se vérifie à la main avant chaque version et se consigne dans une note de `docs/research/`.
- Si une version dépasse le plafond, le choix est le même qu'aujourd'hui : réduire (écran par écran), réviser l'ADR avec la mesure, ou rouvrir l'interface web par un ADR qui remplace l'ADR 0001.

## Impact sur l'architecture

Aucun changement du graphe de modules. G5 repose surtout sur la séparation déjà décidée : un service Breathly léger en arrière-plan, une fenêtre lourde seulement quand l'utilisateur l'ouvre.
