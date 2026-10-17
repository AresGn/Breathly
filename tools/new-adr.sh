#!/usr/bin/env bash
# Crée l'ADR suivant à partir du modèle : tools/new-adr.sh "Titre de la décision"
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
title="${1:?usage : tools/new-adr.sh \"Titre de la décision\"}"
last="$(ls docs/adr | grep -E '^[0-9]{4}-' | sort | tail -1 | cut -c1-4 || true)"
next="$(printf '%04d' $((10#${last:-0000} + 1)))"
slug="$(printf '%s' "$title" | iconv -f utf-8 -t ascii//TRANSLIT 2>/dev/null | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g')"
file="docs/adr/${next}-${slug}.md"
sed -e "s/ADR 0000 : .*/ADR ${next} : ${title}/" -e "s/^- \*\*Date\*\* : .*/- **Date** : $(date +%Y-%m-%d)/" docs/adr/0000-template.md > "$file"
echo "créé : $file (statut « en attente » : à compléter, puis à relier dans docs/README.md si besoin)"
