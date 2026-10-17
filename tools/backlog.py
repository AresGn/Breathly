#!/usr/bin/env python3
"""Génère docs/backlog.md : la vue globale des tickets (assignés, bloqués, à faire en premier).

GitHub reste la source de vérité ; ce fichier en est un instantané, à régénérer avec `make backlog`.
Prérequis : `gh` authentifié. Lecture seule : rien n'est modifié sur GitHub.
"""
import json
import re
import subprocess
import sys
import time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from datetime import date
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
CONF = (ROOT / ".kit.conf").read_text(encoding="utf-8")


def conf(key: str) -> str:
    match = re.search(rf'^{key}="?([^"\n#]*)"?', CONF, re.M)
    if not match:
        sys.exit(f".kit.conf : {key} introuvable")
    return match.group(1).strip()


REPO = conf("REPO")
NAME = conf("PROJECT_NAME")
OUT = ROOT / "docs" / "backlog.md"


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


def label_names(issue: dict) -> set[str]:
    return {label["name"] for label in issue["labels"]}


def app_of(issue: dict) -> str:
    return next((n.removeprefix("app:") for n in label_names(issue) if n.startswith("app:")), "-")


def cell(text: str) -> str:
    return text.replace("|", "\\|")


def link(issue: dict) -> str:
    return f"[#{issue['number']}]({issue['html_url']})"


def main() -> None:
    issues = [i for i in gh_api(f"repos/{REPO}/issues?state=all&per_page=100") if "pull_request" not in i]
    by_number = {i["number"]: i for i in issues}
    with ThreadPoolExecutor(max_workers=8) as pool:
        blockers = dict(zip(by_number, pool.map(lambda n: gh_api(f"repos/{REPO}/issues/{n}/dependencies/blocked_by?per_page=100"), by_number)))

    def open_blockers(number: int) -> list[int]:
        return sorted(b["number"] for b in blockers[number] if by_number.get(b["number"], b)["state"] == "open")

    def is_free(issue: dict) -> bool:
        return issue["state"] == "open" and not open_blockers(issue["number"])

    def who(issue: dict) -> str:
        return ", ".join(f"@{a['login']}" for a in issue["assignees"]) or "personne"

    open_issues = [i for i in issues if i["state"] == "open"]
    milestones = sorted({i["milestone"]["number"]: i["milestone"]["title"] for i in issues if i["milestone"]}.items())

    lines = [
        f"# Backlog de {NAME} : vue globale des tickets",
        "",
        f"Instantané du {date.today().isoformat()}, généré par `make backlog` (`tools/backlog.py`). "
        "**GitHub reste la source de vérité** : en cas de doute, ouvrir le ticket. "
        "Règles de travail : [`CONTRIBUTING.md`](../CONTRIBUTING.md) §2 ; gabarit et étiquettes : [`docs/agents/issue-tracker.md`](agents/issue-tracker.md).",
        "",
        "**Lire ce fichier** : un ticket est **libre** quand tous ses bloqueurs sont fermés. Prendre d'abord les tickets libres de la milestone "
        "la plus basse. `ready-for-human` = décision ou travail humain (ADR, valeurs du client) ; `en-attente-decision` = bloqué par une décision non prise.",
        "",
        "## Vue d'ensemble",
        "",
        "| Milestone | Ouverts | Libres | Bloqués | Fermés |",
        "|---|---|---|---|---|",
    ]
    for number, title in milestones:
        group = [i for i in issues if i["milestone"] and i["milestone"]["number"] == number]
        opened = [i for i in group if i["state"] == "open"]
        free = [i for i in opened if is_free(i)]
        lines.append(f"| {cell(title)} | {len(opened)} | {len(free)} | {len(opened) - len(free)} | {len(group) - len(opened)} |")

    lines += ["", "## À faire en premier (tickets libres)", ""]
    free_all = [i for i in open_issues if is_free(i)]
    by_person: dict[str, list[dict]] = defaultdict(list)
    for issue in free_all:
        by_person[who(issue)].append(issue)
    if not free_all:
        lines.append("Aucun ticket libre : tout est bloqué ou fermé.")
    for person in sorted(by_person):
        lines += [f"### {person}", "", "| Ticket | App | Milestone | Étiquettes utiles |", "|---|---|---|---|"]
        for issue in sorted(by_person[person], key=lambda i: (i["milestone"]["number"] if i["milestone"] else 99, i["number"])):
            ms = issue["milestone"]["title"] if issue["milestone"] else "-"
            useful = ", ".join(sorted(label_names(issue) & {"ready-for-human", "ready-for-agent", "en-attente-decision"}))
            lines.append(f"| {link(issue)} {cell(issue['title'])} | {app_of(issue)} | {cell(ms)} | {useful} |")
        lines.append("")

    lines += ["## Décisions qui attendent le propriétaire ou le client", "", "| Ticket | Assigné | Débloque |", "|---|---|---|"]
    for issue in sorted(open_issues, key=lambda i: i["number"]):
        if "ready-for-human" not in label_names(issue):
            continue
        unlocks = sorted(n for n, bs in blockers.items() if by_number[n]["state"] == "open" and any(b["number"] == issue["number"] for b in bs))
        shown = ", ".join(f"#{n}" for n in unlocks[:8]) + (" …" if len(unlocks) > 8 else "")
        lines.append(f"| {link(issue)} {cell(issue['title'])} | {who(issue)} | {len(unlocks)} ticket(s) {shown} |")

    for number, title in milestones:
        group = sorted((i for i in issues if i["milestone"] and i["milestone"]["number"] == number), key=lambda i: i["number"])
        lines += ["", f"## {title}", "", "| Ticket | App | Assigné | État | Bloqué par (ouverts) |", "|---|---|---|---|---|"]
        for issue in group:
            if issue["state"] == "closed":
                state, blocked = "fermé", ""
            else:
                pending = open_blockers(issue["number"])
                state = "libre" if not pending else "bloqué"
                blocked = ", ".join(f"[#{n}]({by_number[n]['html_url']})" if n in by_number else f"#{n}" for n in pending)
            lines.append(f"| {link(issue)} {cell(issue['title'])} | {app_of(issue)} | {who(issue)} | {state} | {blocked} |")

    unassigned = [i for i in open_issues if not i["assignees"]]
    lines += ["", "## Tickets sans assigné", ""]
    lines.append(f"{len(unassigned)} ticket(s) ouvert(s) sans assigné : " + (", ".join(link(i) for i in sorted(unassigned, key=lambda i: i['number'])) or "aucun") + ".")
    lines.append("")
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"{OUT} : {len(issues)} tickets, {len(free_all)} libres")


if __name__ == "__main__":
    main()
