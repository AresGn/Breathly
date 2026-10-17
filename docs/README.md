# Documentation du projet

La documentation vit dans le dépôt, en Markdown, et se relit comme le code : une PR qui change une règle change son document dans la **même PR** (`CONTRIBUTING.md` §9, case « Documentation » du modèle de PR). Ce fichier dit où est quoi et qui le met à jour. Un document qui n'a pas de ligne ici n'a pas de propriétaire.

## Où est quoi

| Je cherche… | Document | Mis à jour quand… |
|---|---|---|
| Les règles du dépôt (flux, contrôles, CI locale, revue) | `CONTRIBUTING.md` | une règle de travail change |
| Les invariants métier et le style de code | `AGENTS.md` | un invariant ou un seuil change (court par conception) |
| Écrire du code dans une application | `CODING.md` de chaque application (chemins dans `.kit.conf`) | une convention de code change |
| Le vocabulaire | `GLOSSARY.md` | un terme est ajouté ou précisé |
| Ce que fait le produit (exigences, états, permissions) | `prd/` | une règle métier change |
| Pourquoi on a choisi (décisions) | `adr/` | une décision est prise ou remplacée (jamais réécrite en silence) |
| Comment c'est conçu (système, base, écrans) | `design/` | la conception change |
| Les règles de sécurité par conception | `policies/securite.md` | une règle de sécurité change |
| Le modèle de menaces | `policies/modele-de-menaces.md` | un flux critique change |
| Les politiques de conformité | `policies/` | décision du client ou note de conformité |
| Démarrer son poste, initialiser une application | `guides/demarrage-developpeur.md`, README de chaque application | un outil, une version ou une commande change |
| Les principes de conception | `guides/principes-de-conception.md` | rarement |
| Comment les tickets sont écrits, étiquetés et reliés | `agents/issue-tracker.md`, `agents/triage-labels.md` | le flux de tickets change |
| Qui a quel ticket, ce qui est bloqué, par quoi commencer | `backlog.md` (généré par `make backlog`, instantané ; GitHub fait foi) | à régénérer quand des tickets se ferment ou changent de main |
| Les sources d'une décision (légal, fournisseurs…) | `research/` | une recherche est datée et ajoutée, jamais remplacée |
| Les procédures d'exploitation | `runbooks/` | à la demande, avec le premier besoin réel |
| Le cahier des charges d'origine | `cdc/` | jamais (référence) |
| Le contrat d'API | fichier généré (voir `tools/extra-checks.sh`) | **généré**, jamais écrit à la main |

## Règles de maintenance

1. **Une règle, un seul document.** Les autres y renvoient.
2. **Les décisions sont des ADR** (`tools/new-adr.sh`). Un ADR accepté ne se réécrit pas : on le complète d'une note datée ou on le remplace par un nouvel ADR.
3. **Ce qui est vérifié cite sa source et sa date ; ce qui ne l'est pas le dit** (« non vérifié »).
4. **Diagrammes en Mermaid**, dans le Markdown du document qu'ils illustrent (rendu natif par GitHub). À écrire avec l'implémentation des flux qui ne tolèrent aucune erreur (authentification, fichiers sensibles), pas avant.
5. **Le contrat d'API et les clients ne s'écrivent pas à la main.**
6. **Les runbooks d'exploitation s'écrivent à la demande**, avec le premier besoin réel, dans `runbooks/`. Un runbook écrit avant l'opération est faux dès le déploiement.
7. **Les README des applications** donnent les prérequis exacts, l'installation et les commandes de contrôle avant PR (`make ci-local`). Les comptes de test viennent de fabriques de données dans les tests, jamais d'identifiants réels listés dans un document.
8. `make docs-check` (lancé par la CI) échoue sur un lien relatif cassé ; `make kit-todo` liste ce qu'il reste à remplir après l'initialisation du projet.
