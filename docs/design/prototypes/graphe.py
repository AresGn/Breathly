#!/usr/bin/env python3
"""Prototype JETABLE (étape 2.5) : le graphe de dépendances des modules est-il sans cycle et conforme au PRD §5 ?

  python3 -I docs/design/prototypes/graphe.py

Compare le graphe du design (docs/design/0002-systeme.md §3) au tableau du PRD §5, cherche les cycles
(appels + messages) et les modules sans dépendance déclarée.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

# Appels : « A appelle un cas d'usage public de B ». app-ui appelle tous les autres (PRD §5), traité à part.
CALLS = {
    "platform": [], "rules": [],
    "diagnose": ["platform"],
    "disk-scan": ["platform", "rules"],
    "disk-clean": ["platform", "rules", "disk-scan"],
    "daemon": ["platform"],
    "browser-guard": ["platform", "daemon"],
    "app-watch": ["platform", "daemon"],
    "tab-sleep": ["daemon", "browser-guard"],
    "startup": ["platform"],
    "system-tune": ["platform", "rules"],
}
# Messages : « A envoie un message à B ». Le service ne notifie jamais app-ui (il notifie l'OS), donc aucun message vers app-ui.
MESSAGES: dict[str, list[str]] = {}


def prd_dependencies() -> dict[str, set[str]]:
    deps = {}
    text = (ROOT / "docs" / "PRD.md").read_text(encoding="utf-8")
    section = text[text.index("## 5. Carte des modules"):text.index("**Ordre de construction")]
    for line in section.splitlines():
        m = re.match(r"^\| `([a-z-]+)` \|.*\| ([^|]*) \|$", line)
        if m and m.group(1) in CALLS | {"app-ui": []}:
            deps[m.group(1)] = set(re.findall(r"`([a-z-]+)`", m.group(2)))
    return deps


def find_cycle(graph: dict[str, list[str]]) -> list[str] | None:
    state: dict[str, int] = {}

    def visit(node: str, path: list[str]) -> list[str] | None:
        state[node] = 1
        for nxt in graph.get(node, []):
            if state.get(nxt) == 1:
                return path + [node, nxt]
            if state.get(nxt) is None and (cycle := visit(nxt, path + [node])):
                return cycle
        state[node] = 2
        return None

    for n in graph:
        if state.get(n) is None and (cycle := visit(n, [])):
            return cycle
    return None


def main() -> int:
    ok = True
    union = {n: list(CALLS.get(n, [])) + MESSAGES.get(n, []) for n in set(CALLS) | {"app-ui"}}
    union["app-ui"] = [m for m in CALLS]  # app-ui appelle tous les modules, aucun module ne l'appelle
    for name, graph in (("appels", CALLS), ("messages", MESSAGES), ("union", union)):
        cycle = find_cycle(graph)
        print(f"{name:9} {'sans cycle' if not cycle else 'CYCLE : ' + ' -> '.join(cycle)}")
        ok &= cycle is None
    prd = prd_dependencies()
    for module, deps in CALLS.items():
        if prd.get(module) is None:
            print(f"module absent du PRD §5 : {module}"); ok = False
        elif prd[module] - {"—"} != set(deps):
            print(f"écart PRD §5 / design pour {module} : PRD={sorted(prd[module])} design={sorted(deps)}"); ok = False
    missing = [m for m in prd if m not in CALLS and m != "app-ui"]
    if missing:
        print("modules du PRD absents du design :", missing); ok = False
    print(f"conformité au PRD §5 : {'OK' if ok else 'ÉCHEC'} ({len(CALLS)} modules)")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
