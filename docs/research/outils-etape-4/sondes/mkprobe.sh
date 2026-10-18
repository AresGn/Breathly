#!/usr/bin/env bash
# usage: mkprobe.sh <nom> <rust|ui> <cible make> <jeton|-> <fail|pass>
S=$(dirname "$(readlink -f "$0")"); R=$S/repo-copy
name=$1; kind=$2; target=$3; token=$4; expect=$5
SNAP="crates Cargo.toml Cargo.lock apps/desktop/ui/src apps/desktop/ui/package.json apps/desktop/ui/eslint.config.js"
[ -f $S/snap.tar ] || tar -C $R -cf $S/snap.tar $SNAP
restore() { cd $R && rm -rf $SNAP && tar -xmf $S/snap.tar && rm -rf apps/desktop/ui/src/lib/api apps/desktop/ui/src/*.d.ts.bak; }
restore
export CARGO_TARGET_DIR=$S/target-shared
if [ $kind = rust ]; then . $S/mk-rust-mut.sh; ( cd $R && "mutate_$name" && { [ "$name" = fmt ] || cargo fmt --all; } && cargo metadata --format-version 1 >/dev/null 2>&1 )
else . $S/umutations.sh; ( cd $R/apps/desktop/ui && "umut_$name" && { [ "$name" = prettier ] || pnpm exec prettier --write src >/dev/null 2>&1; } ); fi
cd $R; out=$(make $target 2>&1); rc=$?
if [ "$expect" = fail ]; then
  if [ $rc -eq 0 ]; then v="SONDE NON DÉTECTÉE"
  elif [ "$token" != "-" ] && ! grep -q -- "$token" <<<"$out"; then v="ÉCHEC POUR UNE AUTRE RAISON (jeton '$token' absent)"
  else v="OK make $target échoue (rc=$rc, '$token')"; fi
else [ $rc -eq 0 ] && v="OK accepté" || v="REFUSÉ À TORT (rc=$rc)"; fi
echo "### [$kind] $name : $v"; case "$v" in OK*) ;; *) echo "$out" | tail -n 60;; esac
restore
