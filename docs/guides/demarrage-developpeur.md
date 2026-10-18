# Guide de démarrage développeur

Pour qu'une personne qui rejoint le projet (ou un agent) puisse initialiser son poste, comprendre où mettre le code et savoir quoi lire. Court par conception : chaque application a son propre `README.md` pour le détail.

Dernière vérification des versions : 2026-10-18 (poste du propriétaire : Linux Mint, Rust 1.97.1, Node 24.16.0, pnpm 11.5.3). Les versions exactes du projet se fixent à l'étape 5 (`rust-toolchain.toml`, `packageManager`).

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
| Rust (via `rustup`) | version fixée par `rust-toolchain.toml` (1.97.1 à ce jour) | cœur, application Tauri |
| Node.js (LTS) et pnpm | Node 24.16.0 et pnpm 11.5.3 (éprouvés le 2026-10-18) ; `packageManager` à fixer à l'étape 5 | interface Svelte (et extension en V0.3) |
| Linux : paquets de développement de Tauri | `sudo apt update` puis `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` | construire l'application (liste de la page officielle Tauri v2 pour Debian, lue le 2026-10-18) |
| Windows : Microsoft C++ Build Tools (« Desktop development with C++ ») et WebView2 | WebView2 est déjà présent sur Windows 10 (1803) et suivants | construire l'application |
| `cargo-audit` et `cargo-deny` | `cargo install cargo-audit --version 0.22.2 --locked` et `cargo install cargo-deny --version 0.20.2 --locked` (éprouvés le 2026-10-18) | `make audit` (avis de sécurité, demande Internet) et `make check` (bibliothèques réseau interdites, sens des dépendances) |

Windows : l'application se compile avec les outils **natifs** (MSVC, WebView2) ; les scripts et le `Makefile` sont en bash (Git Bash ou WSL 2). Compilation sous Windows : non vérifiée à ce jour, à essayer sur la machine de validation. Ajouter une règle TOML ou une traduction ne demande **aucun** outil de compilation.

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
| `.` (cœur Rust, `crates/`) | pas encore initialisé | une personne crée le workspace et les 4 crates de la V0.1 (étape 5) | `CODING.md` |
| `apps/desktop/ui` (interface Svelte) | pas encore initialisé | après le workspace : coquille Tauri + SvelteKit (étape 5) | `apps/desktop/ui/README.md` |

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
