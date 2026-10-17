#!/usr/bin/env bash
# CI locale : rejoue tout ce que rejoue le job « gate » de la CI, puis publie le statut `local-ci` sur le commit.
#   tools/ci-local.sh               contrôles complets, puis publication du statut si tout est vert
#   tools/ci-local.sh --no-publish  contrôles complets, sans publication (essai à blanc)
# Prérequis : arbre propre, commit déjà poussé, `gh` authentifié (pour publier).
# Sert de porte de fusion quand la CI hébergée n'est pas disponible (minutes épuisées, offre gratuite).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# shellcheck source=/dev/null
. ./.kit.conf

publish=1
case "${1:-}" in
  "") ;;
  --no-publish) publish=0 ;;
  *) echo "usage : $0 [--no-publish]" >&2; exit 2 ;;
esac

fail() { echo "ci-local : $1" >&2; exit 1; }

[ -z "$(git status --porcelain)" ] || fail "l'arbre de travail n'est pas propre (commiter ou remiser d'abord)"

base="origin/${DEFAULT_BRANCH}"
git rev-parse --verify -q "$base" >/dev/null || fail "$base introuvable (git fetch origin ${DEFAULT_BRANCH})"
sha="$(git rev-parse HEAD)"
[ -n "$(git branch -r --contains "$sha")" ] || fail "le commit $sha n'est pas poussé (git push d'abord)"

echo "ci-local : secrets (fichiers suivis, puis historique $base..HEAD)"
bash tools/scan-secrets.sh --all
bash tools/scan-secrets.sh --range "$base..HEAD"

echo "ci-local : audit des dépendances"
bash tools/stack.sh audit

echo "ci-local : toutes les applications"
bash tools/stack.sh check

if [ -x tools/extra-checks.sh ]; then
  echo "ci-local : contrôles propres au projet"
  bash tools/extra-checks.sh --full
fi

echo "ci-local : documentation (liens relatifs, actions épinglées)"
python3 -I tools/check-docs.py
python3 -I tools/pin-actions.py --check

[ -z "$(git status --porcelain)" ] || fail "les contrôles ont modifié des fichiers (contrat généré, format ?) : les commiter et relancer"

if [ "$publish" = 0 ]; then
  echo "ci-local : OK (rien publié, --no-publish)"
  exit 0
fi

who="$(gh api user --jq .login)"
gh api -X POST "repos/$REPO/statuses/$sha" \
  -f state=success -f context=local-ci \
  -f description="make ci-local vert ($who, $(date -u +%Y-%m-%dT%H:%MZ))" >/dev/null
echo "ci-local : OK, statut local-ci publié sur ${sha:0:7}"
