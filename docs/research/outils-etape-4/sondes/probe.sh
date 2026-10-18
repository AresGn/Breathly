#!/usr/bin/env bash
# usage: probe.sh <name> <jeton attendu dans la sortie | -> <fail|pass> <cmd...>
S=$(dirname "$(readlink -f "$0")")
name=$1; token=$2; expect=$3; shift 3
W=$S/work-$name; rm -rf "$W"; mkdir "$W"
tar -C "$S/probe-rust" --exclude=target -cf - . | tar -C "$W" -xf -
export CARGO_TARGET_DIR=$S/target-shared
. "$S/mutations.sh"
( cd "$W" && "mutate_$name" && cargo metadata --format-version 1 >/dev/null 2>&1 )
cd "$W"
out=$("$@" 2>&1); rc=$?
if [ "$expect" = fail ]; then
  if [ $rc -eq 0 ]; then v="SONDE NON DÉTECTÉE (rc=0)"
  elif [ "$token" != "-" ] && ! grep -q -- "$token" <<<"$out"; then v="ÉCHEC POUR UNE AUTRE RAISON (jeton '$token' absent)"
  else v="OK détectée (rc=$rc, jeton '$token')"; fi
else
  [ $rc -eq 0 ] && v="OK accepté (rc=0)" || v="REFUSÉ À TORT (rc=$rc)"
fi
echo "### $name : $v"
case "$v" in OK*) ;; *) echo "$out" | grep -E "^error" | head -5;; esac
