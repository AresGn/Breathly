#!/usr/bin/env python3
"""Contrôle de la documentation : liens relatifs cassés entre fichiers Markdown.

  python3 -I tools/check-docs.py          échoue si un lien relatif pointe vers un fichier absent
  python3 -I tools/check-docs.py --todo   liste en plus les marqueurs « À REMPLIR » restants (information)

Ignore : liens http(s), mailto, ancres seules, blocs de code. Lit les fichiers suivis par git.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
FENCE = re.compile(r"```.*?```", re.S)
INLINE = re.compile(r"`[^`\n]*`")
LINK = re.compile(r"\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")


def tracked_markdown() -> list[Path]:
    out = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.md"], capture_output=True, text=True, check=True, cwd=ROOT).stdout
    return [ROOT / line for line in out.splitlines() if line and not line.startswith((".agents/", ".claude/"))]


def main() -> int:
    todo = "--todo" in sys.argv
    broken, placeholders = [], 0
    for md in tracked_markdown():
        if not md.exists():
            continue
        text = md.read_text(encoding="utf-8", errors="replace")
        placeholders += text.count("À REMPLIR")
        body = INLINE.sub("", FENCE.sub("", text))
        for target in LINK.findall(body):
            if target.startswith(("http://", "https://", "mailto:", "#", "file://")):
                continue
            path = target.split("#", 1)[0]
            if not path:
                continue
            resolved = (ROOT / path.lstrip("/")) if path.startswith("/") else (md.parent / path)
            if not resolved.exists():
                broken.append(f"{md.relative_to(ROOT)} -> {target}")
    for line in broken:
        print(f"lien cassé : {line}")
    if todo:
        print(f"marqueurs « À REMPLIR » restants : {placeholders}")
    print(f"check-docs : {len(broken)} lien(s) cassé(s)")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
