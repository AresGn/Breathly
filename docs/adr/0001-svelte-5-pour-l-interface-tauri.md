# ADR 0001 : Svelte 5 pour l'interface Tauri

- **Statut** : accepté, sous réserve du prototype d'un écran (voir Conséquences)
- **Date** : 2026-10-17
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

L'application est en Tauri v2 (PRD §10) : le cœur est en Rust, l'interface est une page web affichée par le moteur web du système. Il faut choisir comment l'écrire. Contraintes du PRD : binaire léger, interface accessible, FR et EN dès la V0.1, thème clair/sombre, et surtout contributeurs web nombreux et peu familiers de Rust (G6). Question ouverte 1 du PRD §17, tranchée le 2026-10-17.

Non vérifié à cette date : la compatibilité et le gabarit officiel Svelte 5 avec Tauri v2, le poids réel du bundle. À vérifier par le prototype avant d'écrire plus d'un écran.

## Options

1. **Svelte 5** : peu de JavaScript embarqué, syntaxe proche du HTML, courbe d'apprentissage courte pour des contributeurs occasionnels. Coût : écosystème de composants plus petit que React.
2. **SolidJS** : réactivité fine, très performant. Coût : communauté plus petite, moins de contributeurs potentiels.
3. **Vanilla + Web Components** : zéro dépendance. Coût : plus de code à écrire et à maintenir pour l'accessibilité, l'i18n et le thème.

## Décision

Svelte 5, en TypeScript. Raison : le critère déterminant est la facilité pour la communauté de contribuer à l'interface sans écrire de Rust.

## Conséquences

- `apps/desktop/ui/` est une application Svelte 5 + TypeScript, en SvelteKit mode SPA (ADR 0010). Elle sera déclarée comme application du kit (`.kit.conf`) avec la pile `node-api` (pnpm) à l'ouverture du premier ticket de l'interface.
- L'interface n'a aucune règle métier : elle appelle les modules par des commandes Tauri (PRD §5, AGENTS.md).
- Le ticket du prototype d'un écran (accueil) porte la vérification : démarrage avec Tauri v2, taille du bundle, i18n FR/EN, thème clair/sombre. S'il échoue, cet ADR est remplacé.

## Revue après mesure (2026-10-18)

La coquille mesurée pèse 288 Mo de PSS en trois processus, contre 96 Mo pour une fenêtre native minimale (egui, un processus). Le propriétaire a décidé de **conserver Tauri + Svelte pour la V0.1** (ADR 0008) : l'interface ne parle au cœur que par des commandes et reste remplaçable.

## Impact sur l'architecture

Module `app-ui` uniquement. Le cœur Rust et l'extension ne sont pas concernés ; changer de framework plus tard ne touche que `apps/desktop/ui/`.
