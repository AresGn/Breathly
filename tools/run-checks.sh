#!/usr/bin/env bash
# Lance les contrôles qui correspondent aux fichiers modifiés. Utilisé par les hooks git.
#   tools/run-checks.sh --staged          fichiers indexés (pré-commit)
#   tools/run-checks.sh --since <réf>     fichiers modifiés depuis <réf> (pré-push)
# Les secrets sont toujours contrôlés. La CI rejoue tout, quel que soit le périmètre.
# Contrôles propres au projet (base de données, contrat d'API…) : tools/extra-checks.sh (voir extra-checks.sh.example).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# shellcheck source=/dev/null
. ./.kit.conf

case "${1:-}" in
  --staged) paths="$(git diff --cached --name-only --diff-filter=ACMRD)" ;;
  --since)  paths="$(git diff --name-only "${2:?--since exige une référence}"...HEAD)" ;;
  *) echo "usage : $0 --staged | --since <réf>" >&2; exit 2 ;;
esac

has() { printf '%s\n' "$paths" | grep -qE "$1"; }

echo "contrôles : secrets"
bash tools/scan-secrets.sh --all

# Outillage commun : tout est concerné.
all=0
has '^(Makefile|tools/|\.githooks/|\.github/|\.kit\.conf)' && all=1

ran=0
for entry in $APPS; do
  path="${entry%%:*}"
  if [ "$all" = 1 ] || has "^${path}/"; then
    bash tools/stack.sh check "$path"
    ran=1
  fi
done

if [ -x tools/extra-checks.sh ]; then
  bash tools/extra-checks.sh "$paths"
fi

if [ "$ran" = 0 ]; then echo "contrôles : aucun code d'application touché, secrets seulement"; fi
exit 0
