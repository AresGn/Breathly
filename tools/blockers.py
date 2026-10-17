#!/usr/bin/env python3
"""Classe les décisions ouvertes par le nombre de tickets qu'elles bloquent (transitif), et dit qui peut avancer sans elles.

  python3 -I tools/blockers.py            tableau dans le terminal
  python3 -I tools/blockers.py --json     sortie JSON

Une « décision » est un ticket ouvert portant `ready-for-human`. Lecture seule : demande `gh` authentifié.
"""
import json
import re
import subprocess
import sys
import time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
REPO = re.search(r'^REPO="?([^"\n#]*)"?', (ROOT / ".kit.conf").read_text(encoding="utf-8"), re.M).group(1).strip()


def gh_api(path: str) -> list | dict:
    """Appelle `gh api` avec 3 essais (une coupure réseau ponctuelle ne doit pas faire échouer l'outil)."""
    result = None
    for attempt in range(3):
        result = subprocess.run(["gh", "api", "--paginate", path], capture_output=True, text=True, check=False)
        if result.returncode == 0:
            break
        time.sleep(1 + attempt)
    if result is None or result.returncode != 0:
        sys.exit(f"gh api {path} a échoué : {result.stderr.strip() if result else 'aucune réponse'}")
    text = result.stdout.strip()
    if text.startswith("["):
        return [item for part in json.loads("[" + text.replace("][", "],[") + "]") for item in part]
    return json.loads(text)


def main() -> None:
    issues = [i for i in gh_api(f"repos/{REPO}/issues?state=all&per_page=100") if "pull_request" not in i]
    by = {i["number"]: i for i in issues}
    with ThreadPoolExecutor(max_workers=8) as pool:
        blocked_by = dict(zip(by, pool.map(lambda n: [b["number"] for b in gh_api(f"repos/{REPO}/issues/{n}/dependencies/blocked_by?per_page=100")], by)))
    opened = {n for n, i in by.items() if i["state"] == "open"}

    def ancestors(n: int, seen: set[int] | None = None) -> set[int]:
        seen = seen if seen is not None else set()
        for b in blocked_by[n]:
            if b in opened and b not in seen:
                seen.add(b)
                ancestors(b, seen)
        return seen

    def labels(n: int) -> set[str]:
        return {label["name"] for label in by[n]["labels"]}

    def owner(n: int) -> str:
        return ",".join(a["login"] for a in by[n]["assignees"]) or "personne"

    decisions = sorted(n for n in opened if "ready-for-human" in labels(n))
    work = [n for n in opened if "ready-for-human" not in labels(n)]
    rows = []
    for d in decisions:
        down = [n for n in work if d in ancestors(n)]
        per_person: dict[str, int] = defaultdict(int)
        for n in down:
            per_person[owner(n)] += 1
        rows.append({"decision": d, "titre": by[d]["title"], "assigne": owner(d), "bloque": len(down), "par_personne": dict(per_person), "premiers": sorted(down)[:6]})
    rows.sort(key=lambda r: -r["bloque"])
    free = [n for n in work if not ancestors(n)]
    without_decision = [n for n in work if not (ancestors(n) & set(decisions))]

    if "--json" in sys.argv:
        print(json.dumps({"decisions": rows, "libres": sorted(free), "sans_decision": len(without_decision), "tickets_de_travail": len(work)}, ensure_ascii=False, indent=1))
        return
    print(f"{len(work)} tickets de travail ouverts ; {len(without_decision)} n'attendent aucune décision ; {len(free)} sont libres maintenant.\n")
    print(f"{'#':>5}  {'bloque':>6}  {'assigné':16} décision")
    for r in rows:
        print(f"{r['decision']:>5}  {r['bloque']:>6}  {r['assigne']:16} {r['titre'][:70]}  {r['par_personne'] or ''}")
    unassigned = [r for r in rows if r["assigne"] == "personne" and r["bloque"]]
    if unassigned:
        print("\nATTENTION : décision(s) sans assigné qui bloquent du travail :", ", ".join(f"#{r['decision']}" for r in unassigned))


if __name__ == "__main__":
    main()
