#!/usr/bin/env python3
"""Publie un lot de tickets (une vague) sur GitHub, dans l'ordre des dépendances, avec liens de blocage natifs.

  python3 -I tools/publish-tickets.py <dossier> <fichier.json> --dry-run   valide et affiche l'ordre, ne publie rien
  python3 -I tools/publish-tickets.py <dossier> <fichier.json>             publie

Format du JSON : liste d'objets {code, title, app, state, extra, blockers[], body, category[, milestone][, assignee]}.
- `code` : identifiant local du brouillon (ex. A01, M03). Dans les corps, `(A01)` est remplacé par `(#numéro)` après création.
- `blockers` : une ligne par bloqueur, contenant `(CODE)` d'un ticket du lot ou `(#numéro)` d'un ticket existant, ou « Aucun ».
- `state` : ready-for-agent | ready-for-human ; `extra` : étiquette supplémentaire éventuelle (ex. en-attente-decision).
Le corps doit contenir une section « ## Bloqué par » identique à `blockers` (contrôlé).
Prérequis : `gh` authentifié, étiquettes créées (tools/init-labels.sh), milestones créées si utilisées.
Écrit <dossier>/_numeros-publies.json (code -> numéro) ; refuse de republier un lot déjà publié.
"""
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
REPO = re.search(r'^REPO="?([^"\n#]*)"?', (ROOT / ".kit.conf").read_text(encoding="utf-8"), re.M).group(1).strip()
CODE_REF = re.compile(r"\(([A-Z]+\d+)\)")
NUM_REF = re.compile(r"\(#(\d+)\)")


def gh(*args: str) -> str:
    result = subprocess.run(["gh", *args], capture_output=True, text=True)
    if result.returncode:
        sys.exit(f"ERREUR gh {' '.join(args[:4])} : {result.stderr.strip()}")
    return result.stdout.strip()


def validate(tickets: dict[str, dict]) -> list[str]:
    errors: list[str] = []
    codes = set(tickets)
    for code, t in tickets.items():
        for key in ("title", "app", "state", "blockers", "body", "category"):
            if not t.get(key):
                errors.append(f"{code} : champ « {key} » manquant")
        if t["state"] not in ("ready-for-agent", "ready-for-human"):
            errors.append(f"{code} : état inconnu {t['state']}")
        section = t["body"].split("## Bloqué par\n")
        if len(section) < 2:
            errors.append(f"{code} : section « Bloqué par » absente du corps")
        else:
            lines = [s[2:] for s in section[1].split("\n## ")[0].strip().split("\n") if s.startswith("- ")]
            if lines != t["blockers"]:
                errors.append(f"{code} : le corps et `blockers` diffèrent")
        for line in t["blockers"]:
            refs = CODE_REF.findall(line)
            if not refs and not NUM_REF.search(line) and "Aucun" not in line:
                errors.append(f"{code} : bloqueur sans référence : {line}")
            errors += [f"{code} : code inconnu {r}" for r in refs if r not in codes]
    return errors


def order_of(tickets: dict[str, dict], deps: dict[str, list[str]]) -> list[str]:
    order: list[str] = []
    seen: set[str] = set()
    stack: set[str] = set()

    def visit(code: str) -> None:
        if code in stack:
            sys.exit(f"cycle de dépendances autour de {code}")
        if code in seen:
            return
        seen.add(code)
        stack.add(code)
        for dep in deps[code]:
            visit(dep)
        stack.discard(code)
        order.append(code)

    for code in tickets:
        visit(code)
    return order


def main() -> None:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    dry = "--dry-run" in sys.argv
    if len(args) != 2:
        sys.exit(__doc__)
    folder, name = Path(args[0]), args[1]
    tickets = {t["code"]: t for t in json.loads((folder / name).read_text(encoding="utf-8"))}
    errors = validate(tickets)
    if errors:
        sys.exit("validation échouée :\n  " + "\n  ".join(errors))
    deps = {c: [m for line in t["blockers"] for m in CODE_REF.findall(line) if m in tickets] for c, t in tickets.items()}
    existing = {c: sorted({int(m) for line in t["blockers"] for m in NUM_REF.findall(line)}) for c, t in tickets.items()}
    order = order_of(tickets, deps)
    published = folder / "_numeros-publies.json"
    if dry:
        for c in order:
            t = tickets[c]
            labels = [t["category"], "app:" + t["app"], t["state"]] + ([t["extra"]] if t["extra"] else [])
            print(f"{c:6} {t['title'][:60]:60} {','.join(labels)}  bloqué par {deps[c] + [f'#{n}' for n in existing[c]] or 'aucun'}")
        print(f"validation OK : {len(order)} tickets, aucun cycle")
        return
    if published.exists():
        sys.exit(f"{published} existe : ce lot est déjà publié (supprimer le fichier pour forcer)")

    numbers: dict[str, int] = {}

    def render(body: str) -> str:
        return CODE_REF.sub(lambda m: f"(#{numbers[m.group(1)]})" if m.group(1) in numbers else m.group(0), body)

    tmp = Path(tempfile.mkdtemp(prefix="publish-tickets-"))
    for c in order:
        t = tickets[c]
        labels = [t["category"], "app:" + t["app"], t["state"]] + ([t["extra"]] if t["extra"] else [])
        body_file = tmp / f"{c}.md"
        body_file.write_text(render(t["body"]), encoding="utf-8")
        cmd = ["issue", "create", "--repo", REPO, "--title", t["title"], "--body-file", str(body_file), "--label", ",".join(labels)]
        if t.get("milestone"):
            cmd += ["--milestone", t["milestone"]]
        if t.get("assignee"):
            cmd += ["--assignee", t["assignee"]]
        numbers[c] = int(gh(*cmd).rsplit("/", 1)[1])
        print(f"{c} -> #{numbers[c]}", flush=True)
    published.write_text(json.dumps(numbers, indent=1), encoding="utf-8")

    # Deuxième passage : renvois vers des tickets créés après celui qui les cite.
    for c in order:
        new = render(tickets[c]["body"])
        if new != (tmp / f"{c}.md").read_text(encoding="utf-8"):
            f = tmp / f"{c}.v2.md"
            f.write_text(new, encoding="utf-8")
            gh("issue", "edit", str(numbers[c]), "--repo", REPO, "--body-file", str(f))
            print(f"corrigé {c} #{numbers[c]}")

    ids: dict[int, str] = {}

    def db_id(n: int) -> str:
        if n not in ids:
            ids[n] = gh("api", f"repos/{REPO}/issues/{n}", "--jq", ".id")
        return ids[n]

    for c in order:
        for blocker in sorted({numbers[d] for d in deps[c]} | set(existing[c])):
            gh("api", "--method", "POST", f"repos/{REPO}/issues/{numbers[c]}/dependencies/blocked_by", "-F", f"issue_id={db_id(blocker)}")
        print(f"liens {c} #{numbers[c]}", flush=True)


if __name__ == "__main__":
    main()
