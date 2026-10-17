#!/usr/bin/env bash
# Exécute une action sur les applications déclarées dans .kit.conf (APPS="chemin:pile ...").
#   tools/stack.sh <action> [chemin]
# Actions : setup, lint, fix, static, arch, test, audit, check (= lint + static + arch + test).
# Une application dont le fichier marqueur (STACK_MARKER de sa pile) n'existe pas encore est sautée : on peut
# déclarer une application avant de l'initialiser sans casser la CI.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
# shellcheck source=/dev/null
. ./.kit.conf

action="${1:?usage : tools/stack.sh <setup|lint|fix|static|arch|test|audit|check> [chemin]}"
only="${2:-}"
status=0

for entry in $APPS; do
  path="${entry%%:*}"
  stack="${entry##*:}"
  [ -z "$only" ] || [ "$only" = "$path" ] || continue
  # Le sous-shell n'est pas placé dans une condition (|| ou if) : `set -e` y reste actif, une porte qui échoue arrête l'application.
  set +e
  (
    set -e
    # Valeurs par défaut : une pile qui ne définit pas une action la saute en le disant.
    for fn in setup lint fix static arch test audit; do
      eval "stack_$fn() { echo \"$path ($stack) : action '$fn' non définie pour cette pile, ignorée\"; }"
    done
    # shellcheck source=/dev/null
    . "tools/stacks/$stack.sh"
    if [ ! -e "$path/$STACK_MARKER" ]; then
      echo "$path ($stack) : pas encore initialisé ($STACK_MARKER absent), rien à faire"
      exit 0
    fi
    echo "== $path ($stack) : $action"
    cd "$path"
    case "$action" in
      check) stack_lint; stack_static; stack_arch; stack_test ;;
      setup | lint | fix | static | arch | test | audit) "stack_$action" ;;
      *) echo "action inconnue : $action" >&2; exit 2 ;;
    esac
  )
  rc=$?
  set -e
  [ "$rc" -eq 0 ] || status=1
done
exit "$status"
