.PHONY: help hooks setup lint fix static arch test audit secrets check ci-local backlog blockers docs-check kit-todo labels pin-actions

# Le Makefile est volontairement mince : la logique vit dans tools/ et dans les piles (tools/stacks/<pile>.sh).
# Les applications et leur pile sont déclarées dans .kit.conf (APPS).

help: ## Liste les commandes
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-12s %s\n", $$1, $$2}'

hooks: ## Active les hooks git (une fois par clone) : pré-commit et pré-push exigent des contrôles verts
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit .githooks/pre-push tools/*.sh

setup: hooks ## Active les hooks et installe les dépendances de chaque application initialisée
	bash tools/stack.sh setup

lint: ## Style (formateur et linter de chaque pile)
	bash tools/stack.sh lint

fix: ## Corrige le style automatiquement
	bash tools/stack.sh fix

static: ## Analyse statique et types
	bash tools/stack.sh static

arch: ## Frontières d'architecture (imports interdits entre couches ou modules)
	bash tools/stack.sh arch

test: ## Tests
	bash tools/stack.sh test

audit: ## Audit des dépendances (vulnérabilités connues)
	bash tools/stack.sh audit

secrets: ## Cherche des secrets dans tous les fichiers suivis
	bash tools/scan-secrets.sh --all

check: secrets ## Toutes les portes de chaque application (lint, statique, architecture, tests)
	bash tools/stack.sh check

ci-local: ## CI locale : rejoue tout le job gate, puis publie le statut local-ci sur le commit (arbre propre, commit poussé)
	bash tools/ci-local.sh

backlog: ## Régénère docs/backlog.md : vue globale des tickets (lecture seule, demande `gh`)
	python3 -I tools/backlog.py

blockers: ## Classe les décisions ouvertes par le nombre de tickets qu'elles bloquent (lecture seule, demande `gh`)
	python3 -I tools/blockers.py

docs-check: ## Liens relatifs de la documentation
	python3 -I tools/check-docs.py

kit-todo: ## Liste les marqueurs « À REMPLIR » restants (adoption du kit)
	@grep -rn "À REMPLIR" --include='*.md' --include='.kit.conf' . 2>/dev/null | grep -v '^./.git/' || echo "Aucun marqueur restant."

labels: ## Crée les étiquettes standard sur GitHub (idempotent)
	bash tools/init-labels.sh

pin-actions: ## Épingle les actions GitHub par SHA (chaîne d'approvisionnement)
	python3 -I tools/pin-actions.py
