#!/usr/bin/env bash
# usage: uprobe.sh <nom> <script pnpm> <jeton attendu|-> <fail|pass>  ; la mutation = fonction umut_<nom>
S=$(dirname "$(readlink -f "$0")")
name=$1; script=$2; token=$3; expect=$4
cd "$S/probe-ui" && git reset -q --hard && git clean -qfd
cd apps/desktop/ui
. "$S/umutations.sh"
"umut_$name"
[ "$name" = prettier ] || pnpm exec prettier --write src >/dev/null 2>&1
out=$(pnpm "$script" 2>&1); rc=$?
if [ "$expect" = fail ]; then
  if [ $rc -eq 0 ]; then v="SONDE NON DÉTECTÉE (rc=0)"
  elif [ "$token" != "-" ] && ! grep -q -- "$token" <<<"$out"; then v="ÉCHEC POUR UNE AUTRE RAISON (jeton '$token' absent)"
  else v="OK détectée (rc=$rc, jeton '$token')"; fi
else
  [ $rc -eq 0 ] && v="OK accepté (rc=0)" || v="REFUSÉ À TORT (rc=$rc)"
fi
echo "### $name [$script] : $v"
case "$v" in OK*) ;; *) echo "$out" | tail -n 12;; esac
cd "$S/probe-ui" && git reset -q --hard && git clean -qfd
