#!/usr/bin/env python3
"""Gardes du code Rust que ni rustc ni clippy ne contrôlent (CODING.md, ADR 0011).

  python3 -I tools/check-rust.py    échoue si une garde est violée

  1. aucun crate sans table de lints : `[lints] workspace = true`, sauf `breathly-platform` qui a la sienne ;
  2. la table de `breathly-platform` est celle du workspace, sauf `unsafe_code` ("deny" au lieu de "forbid") ;
  3. aucun `#[ignore]` : un test ne se saute pas (CONTRIBUTING règle 6) ;
  4. aucun `cfg(target_os = ...)`, `cfg(windows)` ni `cfg(unix)` hors de `breathly-platform` (AGENTS.md).

Lit les fichiers suivis par git. Sans Cargo.toml à la racine (workspace pas encore créé), ne fait rien.
"""
import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
PLATFORM = "crates/breathly-platform"
IGNORE_ATTR = re.compile(r"#\s*\[\s*ignore\b")
# « cfg(windows) », « cfg!(unix) », « cfg_attr(windows, …) », « any/all/not(windows) » et toute forme « target_os = … ».
OS_CFG = re.compile(r"\b(?:target_os|target_family)\s*=|\b(?:cfg|cfg_attr|any|all|not)!?\s*\(\s*(?:windows|unix)\s*[,)]")


def tracked(pattern: str) -> list[Path]:
    out = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", pattern], capture_output=True, text=True, check=True, cwd=ROOT).stdout
    return [ROOT / line for line in out.splitlines() if line]


def load(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def lint_problems(manifest: Path, workspace_lints: dict) -> list[str]:
    where = manifest.parent.relative_to(ROOT).as_posix()
    lints = load(manifest).get("lints")
    if where != PLATFORM:
        return [] if lints == {"workspace": True} else [f"{where} : doit déclarer `[lints] workspace = true`"]
    if not lints or "workspace" in lints:
        return [f"{where} : doit déclarer sa propre table [lints] (unsafe_code = \"deny\")"]
    expected = {**workspace_lints, "rust": {**workspace_lints.get("rust", {}), "unsafe_code": "deny"}}
    return [] if lints == expected else [f"{where} : la table [lints] diffère de celle du workspace (seul unsafe_code change)"]


def source_problems(source: Path) -> list[str]:
    where = source.relative_to(ROOT).as_posix()
    problems = []
    for number, line in enumerate(source.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        code = line.split("//", 1)[0]
        if IGNORE_ATTR.search(code):
            problems.append(f"{where}:{number} : #[ignore] interdit, un test ne se saute pas")
        if OS_CFG.search(code) and not where.startswith(PLATFORM + "/"):
            problems.append(f"{where}:{number} : cfg selon l'OS interdit hors de breathly-platform")
    return problems


def main() -> int:
    root_manifest = ROOT / "Cargo.toml"
    if not root_manifest.exists():
        print("check-rust : pas de workspace Cargo, rien à faire")
        return 0
    workspace_lints = load(root_manifest).get("workspace", {}).get("lints", {})
    problems = []
    for manifest in tracked("crates/*/Cargo.toml") + tracked("apps/desktop/src-tauri/Cargo.toml"):
        problems += lint_problems(manifest, workspace_lints)
    for source in tracked("crates/*.rs") + tracked("apps/desktop/src-tauri/*.rs"):
        problems += source_problems(source)
    for problem in problems:
        print(f"check-rust : {problem}")
    print(f"check-rust : {len(problems)} problème(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
