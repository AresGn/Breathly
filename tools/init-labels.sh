#!/usr/bin/env bash
# Crée sur GitHub les étiquettes standard du kit (idempotent : une étiquette existante est mise à jour).
#   tools/init-labels.sh            crée / met à jour
#   tools/init-labels.sh --dry-run  affiche seulement
# Étiquettes : une par application (app:<nom>), les états de triage, et en-attente-decision.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# shellcheck source=/dev/null
. ./.kit.conf
dry=0; [ "${1:-}" = "--dry-run" ] && dry=1

make_label() { # nom couleur description
  if [ "$dry" = 1 ]; then echo "étiquette : $1 ($2) $3"; return; fi
  gh label create "$1" --color "$2" --description "$3" --repo "$REPO" --force >/dev/null
  echo "étiquette : $1"
}

for entry in $APPS; do
  name="$(basename "${entry%%:*}")"
  make_label "app:${name}" "1d76db" "Application ${entry%%:*}"
done
make_label "needs-triage"        "ededed" "Le mainteneur doit évaluer le ticket"
make_label "needs-info"          "d4c5f9" "En attente d'informations"
make_label "ready-for-agent"     "0e8a16" "Entièrement spécifié, prêt pour un agent"
make_label "ready-for-human"     "1d76db" "Décision ou travail humain (ADR, valeurs du client)"
make_label "en-attente-decision" "fbca04" "Bloqué par une décision non prise"
