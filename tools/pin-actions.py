#!/usr/bin/env python3
"""Épingle les actions GitHub par SHA de commit (chaîne d'approvisionnement) et vérifie qu'elles le restent.

  python3 -I tools/pin-actions.py --check    échoue si une action n'est pas épinglée par un SHA de 40 caractères
  python3 -I tools/pin-actions.py            remplace owner/repo@vX par owner/repo@<sha> # vX (demande `gh`)

Règle : une version publiée depuis moins de MIN_RELEASE_AGE_DAYS jours n'est pas épinglée (avertissement).
"""
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
USES = re.compile(r"^(\s*-?\s*uses:\s*)([\w.-]+/[\w./-]+)@([^\s#]+)(\s*#.*)?$")
SHA = re.compile(r"^[0-9a-f]{40}$")


def conf_age_days() -> int:
    text = (ROOT / ".kit.conf").read_text(encoding="utf-8")
    match = re.search(r"^MIN_RELEASE_AGE_DAYS=(\d+)", text, re.M)
    return int(match.group(1)) if match else 7


def gh_json(path: str) -> dict:
    result = subprocess.run(["gh", "api", path], capture_output=True, text=True)
    if result.returncode:
        sys.exit(f"gh api {path} a échoué : {result.stderr.strip()}")
    return json.loads(result.stdout)


def resolve(action: str, ref: str) -> tuple[str, str]:
    """Renvoie (sha du commit, date ISO du commit) pour une étiquette ou une branche."""
    repo = "/".join(action.split("/")[:2])
    try:
        obj = gh_json(f"repos/{repo}/git/ref/tags/{ref}")["object"]
    except SystemExit:
        obj = gh_json(f"repos/{repo}/git/ref/heads/{ref}")["object"]
    if obj["type"] == "tag":  # étiquette annotée : déréférencer
        obj = gh_json(f"repos/{repo}/git/tags/{obj['sha']}")["object"]
    commit = gh_json(f"repos/{repo}/commits/{obj['sha']}")
    return obj["sha"], commit["commit"]["committer"]["date"]


def main() -> int:
    check = "--check" in sys.argv
    min_age = conf_age_days()
    status = 0
    for wf in sorted((ROOT / ".github" / "workflows").glob("*.y*ml")):
        lines = wf.read_text(encoding="utf-8").splitlines()
        changed = False
        for i, line in enumerate(lines):
            m = USES.match(line)
            if not m or m.group(2).startswith("./"):
                continue
            prefix, action, ref, comment = m.groups()
            if SHA.match(ref):
                continue
            if check:
                print(f"{wf.name}:{i + 1} : {action}@{ref} n'est pas épinglée par SHA (python3 -I tools/pin-actions.py)")
                status = 1
                continue
            sha, date = resolve(action, ref)
            age = (datetime.now(timezone.utc) - datetime.fromisoformat(date.replace("Z", "+00:00"))).days
            if age < min_age:
                print(f"AVERTISSEMENT {action}@{ref} : publiée il y a {age} jour(s), moins que {min_age} : version non épinglée")
                status = 1
                continue
            lines[i] = f"{prefix}{action}@{sha} # {ref}"
            changed = True
            print(f"{wf.name}:{i + 1} : {action}@{ref} -> {sha[:12]} ({age} jours)")
        if changed:
            wf.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("pin-actions :", "échec" if status else "OK")
    return status


if __name__ == "__main__":
    sys.exit(main())
