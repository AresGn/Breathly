# Suivi des tickets : GitHub

Les tickets et les specs de ce dépôt sont des issues GitHub (`AresGn/Breathly`). Toutes les opérations passent par la CLI `gh`.

## Conventions

- **Créer** : `gh issue create --title "..." --body "..."` (corps multi-lignes par heredoc). Un lot entier se publie avec `python3 -I tools/publish-tickets.py` (voir le playbook « backlog par vagues » du kit).
- **Lire** : `gh issue view <numéro> --json number,title,body,labels,comments`.
- **Lister** : `gh issue list --state open --json number,title,body,labels,comments`, filtré par `--label` et `--state`.
- **Commenter** : `gh issue comment <numéro> --body "..."`.
- **Étiquettes** : `gh issue edit <numéro> --add-label "..."` / `--remove-label "..."`.
- **Fermer** : `gh issue close <numéro> --comment "..."`.

Le dépôt se déduit de `git remote -v` ; `gh` le fait seul dans un clone.

## Aucune signature

Aucun texte publié (ticket, commentaire, étiquette de triage) ne porte de mention d'outil ou d'agent, y compris l'avertissement que certaines skills ajoutent par défaut. Règle du dépôt : `CONTRIBUTING.md` §11. Cette consigne l'emporte sur le texte des skills.

## Étiquettes propres au projet

| Étiquette | Sens |
|---|---|
| `app:<nom>` | ticket d'une application (`app:core`, `app:desktop`, `app:desktop-ui` pour Breathly) |
| `en-attente-decision` | bloqué par une décision non prise (ADR, choix du client) ; le corps nomme la décision |

**Un ticket porte exactement une étiquette `app:*`**, un rôle de catégorie (`bug` ou `enhancement`) et un état de triage (`docs/agents/triage-labels.md`). Les trois étiquettes `app:*` ont été créées à la main (`gh label create`) : **ne pas lancer `make labels`**, qui dérive le nom du dossier déclaré dans `.kit.conf` et ajouterait `app:.` et `app:ui` ; l'écart est à remonter au kit (le nom devrait pouvoir être explicite).

## Une fonctionnalité, un ticket par application

Une fonctionnalité donne :

1. un ticket **`app:<api>`** de **contrat** (routes, validations, réponses typées, mises à jour du contrat généré ; les routes répondent 501), puis des tickets d'implémentation, bloqués par ce contrat ;
2. un ticket par application cliente, **bloqué par le ticket de contrat** et non par l'implémentation : les clients se développent contre des faux de l'API tant que la route réelle n'existe pas.

Chaque ticket est une tranche verticale **de son application** : de bout en bout dans l'application (domaine, données, présentation, tests), vérifiable seule.

## Milestones : l'ordre du produit

Chaque ticket appartient à une seule milestone (sans échéance tant que l'équipe et les décisions du client ne sont pas connues). Une milestone se termine quand son **critère démontrable** est vrai, pas quand le dernier ticket est fermé.

| Milestone | Critère de fin démontrable |
|---|---|
| M0 · Socle | les applications démarrent, l'interface navigue entre les écrans de V0.1 sur des faux services, le français, l'anglais et les deux thèmes fonctionnent, les commandes Tauri déclarées sont inventoriées et restreintes à la fenêtre principale, et `make ci-local` est vert |
| M1 · Disque | « Analyser mon PC » lance l'analyse du disque ; l'écran montre l'espace libérable classé Sans risque, À vérifier et Décision personnelle, avec les gros fichiers à part et « Ouvrir le dossier » ; un bandeau apparaît sous 2 Go libres ; rien n'est modifiable sur le disque |
| M2 · Nettoyage sûr | depuis l'accueil, on nettoie ce qui est sans risque en 3 clics avec simulation et confirmation ; les éléments à vérifier se choisissent un par un ; les fichiers vont à la corbeille, les garde-fous sont revérifiés pour chaque élément, l'historique permet d'annuler, et le vidage de la corbeille est confirmé à part ; aucun test ne touche au vrai disque |
| M3 · Diagnostic | « Analyser mon PC » affiche l'état du PC (disque, RAM, swap, CPU, chauffe), le top 10 des logiciels, au plus 5 recommandations classées, les processus à examiner sans les mots « virus » ni « sain », et la commande zram à copier sur Debian, Ubuntu et Mint |
| M4 · Livraison | les installeurs `.msi`, `.deb` et `.rpm` sont construits par la CI avec leurs empreintes SHA-256, respectent les budgets de l'ADR 0008, et la V0.1 est validée sur les machines de référence (PRD §15) |

Ordre de travail : M0, puis M1, M2, M3, M4 (décision du 2026-10-18 : le disque et le nettoyage avant le diagnostic, parce que le critère central du PRD §15 est de libérer de l'espace et que les recommandations du diagnostic chiffrent leur gain avec l'analyse du disque). Les blocages natifs des tickets font foi pour l'ordre fin.

### Points ouverts qui n'ont pas de ticket, et pourquoi

| Point | Raison |
|---|---|
| ADR 0009 (définition de « logiciel inutilisé ») | reporté : elle concerne la V0.2 et attend des essais sous Wayland et Windows ; aucun ticket de V0.1 n'en dépend |
| Attestation de provenance des builds (`docs/policies/securite.md` §3) | reportée : sa disponibilité sur ce dépôt n'est pas vérifiée ; à décider avant la première release, sans bloquer le code |
| Signature de code | reportée à la V1.0 (US-400) |
| Mesures sous Windows, installeur `.msi` | pas hors backlog : ticket de validation de M4 |

Les tickets « décision » (ADR, valeurs du client) portent `ready-for-human` et bloquent les tickets qui les attendent (`en-attente-decision`). Un ticket est libre quand ses bloqueurs sont fermés ; prendre d'abord ceux de la milestone en cours.

## Trouver et prendre un ticket

- Mes tickets, par application : `https://github.com/AresGn/Breathly/issues?q=is%3Aopen+label%3Aapp%3A<nom>` ; en ligne de commande `gh issue list --state open --label app:<nom>`.
- Un ticket est **libre** quand il n'a pas d'assigné et que tous ses bloqueurs sont fermés (la page du ticket affiche « Blocked by »). `make backlog` produit la vue globale.
- Le prendre : `gh issue edit <n> --add-assignee @me`, puis une branche `type/<n>-sujet` et **une PR par ticket** (`CONTRIBUTING.md` §2).

## Dépendances (blocage)

Les **dépendances natives** de GitHub sont la représentation de référence. Ajouter un lien : `gh api --method POST repos/AresGn/Breathly/issues/<enfant>/dependencies/blocked_by -F issue_id=<id-base-de-données-du-bloqueur>`, où l'identifiant est numérique (`gh api repos/AresGn/Breathly/issues/<n> --jq .id`), pas le numéro ni le `node_id`. À défaut, une ligne `Bloqué par : #<n>, #<n>` en tête du corps. Un ticket est libre quand tous ses bloqueurs sont fermés.

## Gabarit d'un ticket

Voir `.github/ISSUE_TEMPLATE/ticket.md` (tickets de travail) et `decision.md` (décisions). Sections : Objet (avec la story du PRD), Application, À lire avant de commencer, Critères d'acceptation vérifiables, Hors périmètre, Bloqué par, Terminé quand.

## Quand une skill dit « publier dans le suivi des tickets »

Créer une issue GitHub, avec le gabarit ci-dessus et les étiquettes ci-dessus.

## Quand une skill dit « récupérer le ticket concerné »

Le lire comme indiqué à **Lire**.

## Cartographie (`/wayfinder`)

- **Carte** : une issue unique, étiquette `wayfinder:map`. Tickets enfants en sous-tickets, étiquette `wayfinder:<type>` (`research`, `prototype`, `grilling`, `task`).
- **Prise** : `gh issue edit <n> --add-assignee @me`. **Résolution** : commentaire de la réponse, fermeture, puis pointeur dans « Decisions so far » de la carte.
