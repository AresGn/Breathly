# Guide de démarrage développeur

Pour qu'une personne qui rejoint le projet (ou un agent) puisse initialiser son poste, comprendre où mettre le code et savoir quoi lire. Court par conception : chaque application a son propre `README.md` pour le détail.

Dernière vérification des versions : <!-- À REMPLIR: date -->.

## 1. Où lire quoi

| Je travaille sur | Je lis, dans cet ordre |
|---|---|
| Tout | `AGENTS.md`, `GLOSSARY.md`, ce guide |
| Une application | son `README.md`, son `CODING.md`, les design et ADR cités par son ticket |
| La base de données | le design de la base et l'ADR correspondant |
| Le contrat d'API | le dossier du contrat et son script de génération |

## 2. Prérequis du poste

| Outil | Version | Pour |
|---|---|---|
| Git, `make`, `bash`, Python 3 | récents | tout (hooks, outils de `tools/`) |
| `gh` (CLI GitHub), authentifié | récent | tickets, `make ci-local`, `make backlog` |
| <!-- À REMPLIR: chaînes d'outils des piles (PHP/Composer, Flutter, Node/pnpm, Go, Rust, Python…) --> | | |

Windows : travailler sous WSL 2 (les scripts et le `Makefile` sont en bash).

## 3. Cloner et préparer

```bash
git clone https://github.com/AresGn/Breathly.git && cd breathly
make hooks            # une fois par clone : active le contrôle avant commit et avant push
make help             # liste les commandes
make setup            # installe les dépendances des applications initialisées
```

## 4. Initialiser chaque application

Les applications sont indépendantes. **Une personne initialise, ouvre une PR courte, puis les autres rebasent dessus** : on évite deux fichiers de verrouillage qui se concurrencent.

| Application | État aujourd'hui | Qui fait quoi | Détail |
|---|---|---|---|
| <!-- À REMPLIR: une ligne par application de .kit.conf --> | | | son `README.md` |

## 5. Conventions communes

**Nommage des fichiers** (une règle par langage ; voir les `CODING.md`).

**Git** : une branche par ticket, messages `type(portée): objet`, une PR par ticket. Le détail est dans `CONTRIBUTING.md` (qui fait foi).

**Qualité, ce qui bloque quoi** : voir `CONTRIBUTING.md` §3.

**Décisions** : une décision structurante devient un ADR (`tools/new-adr.sh`). Un point à trancher avec le client reste visible dans la section « Questions ouvertes » du document concerné.

**Secrets** : aucune clé, aucun mot de passe, aucun jeton dans le dépôt. Les clés sont générées par poste et par environnement.

## 6. Agents et skills

Le dépôt peut embarquer des skills pour les agents (Claude Code, Codex…) dans `.agents/skills/` ; `skills-lock.json` fige leur version : **ne pas l'éditer à la main**, il est tenu par l'outil d'installation. Voir `skills/README.md` du kit pour les skills recommandées et le moment où les utiliser.

## 7. Liste de contrôle d'un poste prêt

- [ ] `make hooks` exécuté (sans lui, rien ne vous protège avant la CI)
- [ ] `gh auth status` est vert
- [ ] L'application sur laquelle je travaille démarre (voir son `README.md`)
- [ ] `make check` passe sur ma branche
