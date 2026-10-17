#!/usr/bin/env bash
# Détection locale de secrets (heuristique, sans dépendance). Complète un scanner dédié (GitGuardian, Gitleaks), ne le remplace pas.
#   tools/scan-secrets.sh            fichiers indexés pour le commit (hook de pré-commit)
#   tools/scan-secrets.sh --all      tous les fichiers suivis par git (CI, make secrets)
#   tools/scan-secrets.sh --range A..B   lignes ajoutées par chaque commit de A..B (hook de pré-push, CI sur PR) :
#                                    un secret retiré dans un commit suivant reste dans l'historique de la PR
# Une ligne légitime se marque avec le commentaire « secret-scan: allow » (à justifier en relecture).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

tmp=""
trap '[ -z "$tmp" ] || rm -f "$tmp"' EXIT
case "${1:-}" in
  --all) files="$(git ls-files)" ;;
  --range)
    range="${2:?--range exige une plage git, par exemple origin/main..HEAD}"
    tmp="$(mktemp)"
    # Lignes ajoutées par chaque commit de la plage, y compris celles retirées plus tard.
    git log -p --no-color --no-merges --format= "$range" -- . \
      ':(exclude)*.lock' ':(exclude)pnpm-lock.yaml' ':(exclude)package-lock.json' ':(exclude)go.sum' \
      ':(exclude).agents' ':(exclude).claude' \
      | grep -E '^\+[^+]' | sed 's/^+//' > "$tmp" || true
    files="$tmp" ;;
  *) files="$(git diff --cached --name-only --diff-filter=ACMR)" ;;
esac

# Exclus : verrous de dépendances, skills d'agents, binaires courants.
files="$(printf '%s\n' "$files" | grep -Ev '(^|/)(composer\.lock|pubspec\.lock|pnpm-lock\.yaml|package-lock\.json|yarn\.lock|Cargo\.lock|go\.sum|poetry\.lock|uv\.lock|skills-lock\.json)$|^\.agents/|^\.claude/|\.(png|jpg|jpeg|gif|webp|ico|pdf|ttf|otf|woff2?|jar|keystore)$' || true)"
[ -n "$files" ] || { echo "scan-secrets : rien à analyser."; exit 0; }

status=0

# $1 motif, $2 description, $3 (optionnel) motif des valeurs acceptées (placeholders, variables)
report() {
  local hits
  hits="$(printf '%s\n' "$files" | xargs -d '\n' grep -InEi -e "$1" 2>/dev/null | grep -v 'secret-scan: allow' || true)"
  if [ -n "${3:-}" ] && [ -n "$hits" ]; then
    hits="$(printf '%s\n' "$hits" | grep -Eiv -e "$3" || true)"
  fi
  if [ -n "$hits" ]; then
    echo "scan-secrets : $2"
    printf '%s\n' "$hits" | sed 's/^/  /'
    status=1
  fi
}

report '-----BEGIN ([A-Z]+ )?PRIVATE KEY-----' "clé privée"
report 'AKIA[0-9A-Z]{16}' "identifiant de clé AWS"
report 'gh[pousr]_[A-Za-z0-9]{36,}' "jeton GitHub"
report 'xox[baprs]-[A-Za-z0-9-]{10,}' "jeton Slack"
report 'sk_live_[A-Za-z0-9]{16,}' "clé secrète de paiement"

# Mot de passe, secret, jeton ou clé d'API affecté à une valeur littérale d'au moins 8 caractères.
# Valeurs acceptées : variables ($…), références d'environnement, placeholders (change-me, example, xxx, votre…).
report "(password|passwd|secret|api[_-]?key|token)[A-Za-z0-9_]*[\"']?(:|[[:space:]]*=)[[:space:]]*[\"']?[A-Za-z0-9/+_.-]{8,}" \
  "secret affecté à une valeur littérale" \
  '[:=][[:space:]]*["'"'"']?(\$|\{\{|<|change-me|changeme|example|xxx|your|votre|null|true|false|none)|[:=][[:space:]]*["'"'"']?[a-z_]+\('

if [ "$status" -ne 0 ]; then
  echo "scan-secrets : échec. Retirer la valeur (variable d'environnement, générée par poste) ou, si c'est un faux positif, ajouter « secret-scan: allow » sur la ligne."
fi
exit "$status"
