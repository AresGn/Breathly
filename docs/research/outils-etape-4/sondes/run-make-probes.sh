#!/usr/bin/env bash
S=$(dirname "$(readlink -f "$0")"); cd $S
P=./mkprobe.sh
$P clean rust check - pass
$P fmt rust check "Diff in" fail
$P pedantic rust check must-use-candidate fail
$P unwrap_prod rust check unwrap-used fail
$P cognitive rust check cognitive-complexity fail
$P too_many_lines rust check too-many-lines fail
$P unsafe_rules rust check unsafe-code fail
$P unsafe_allow_in_rules rust check E0453 fail
$P wildcard rust check wildcard-enum-match-arm fail
$P allow_no_reason rust check allow-attributes-without-reason fail
$P process_command rust check disallowed fail
$P test_fail rust check "test result: FAILED\|assertion" fail
$P dep_platform_rules rust check "is explicitly banned" fail
$P dep_diagnose_disk rust check "is explicitly banned" fail
$P dep_ureq rust check "is explicitly banned" fail
$P dep_dev_ureq rust check "is explicitly banned" fail
$P cfg_os rust check "cfg selon l'OS interdit" fail
$P cfg_os_platform rust check - pass
$P ignore_plain rust check "ignore" fail
$P ignore_reason rust check "ignore\] interdit" fail
$P nesting rust check "too nested" fail
$P lints_missing rust check "doit déclarer" fail
$P lints_drift rust check "diffère" fail
$P lints_platform_forbid rust check "diffère" fail
$P unsafe_platform_allowed rust check - pass
$P prettier ui check "Code style issues" fail
$P any ui check no-explicit-any fail
$P ts_ignore ui check ban-ts-comment fail
$P complexity ui check complexity fail
$P switch_missing ui check switch-exhaustiveness-check fail
$P tauri_outside ui check no-restricted-imports fail
$P test_skip ui check no-restricted-properties fail
$P svelte_check_type ui check "not assignable" fail
$P test_fail ui check "Salut" fail
$P color_hardcoded ui check "Col.svelte" fail
echo FIN
