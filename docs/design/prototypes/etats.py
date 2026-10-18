#!/usr/bin/env python3
"""Prototype JETABLE (étape 2.4) : parcourt tous les états atteignables des machines de docs/design/0001-machines-a-etats.md.

  python3 -I docs/design/prototypes/etats.py

Ne sert pas de base au code Rust : il vérifie seulement que les tableaux de transitions n'ont ni état inatteignable,
ni impasse non déclarée, et que la suppression d'un fichier n'a jamais lieu sans entrée d'historique.
"""
import sys
from collections import deque

# machine : (état initial, états terminaux, {état: {événement: état suivant}})
ANALYSE = ("Inactive", {"Done", "Cancelled", "Failed"}, {
    "Inactive": {"start": "Running"},
    "Running": {"finish": "Done", "cancel": "Cancelled", "fatal": "Failed"},
    "Done": {"start": "Running"},
    "Cancelled": {"start": "Running"},
    "Failed": {"start": "Running"},
})
# Les états terminaux d'ANALYSE sont des résultats : on peut relancer (start). Ils ne sont pas des impasses.
NETTOYAGE = ("Draft", {"Completed", "CompletedWithErrors", "Stopped", "Aborted"}, {
    "Draft": {"select": "Draft", "simulate": "Draft", "clean": "Confirming"},
    "Confirming": {"back": "Draft", "confirm": "Running"},
    "Running": {"stop": "Stopping", "all_ok": "Completed", "some_errors": "CompletedWithErrors", "history_unwritable": "Aborted"},
    "Stopping": {"item_done": "Stopped"},
    "Completed": {}, "CompletedWithErrors": {}, "Stopped": {}, "Aborted": {},
})
ENTREE = ("Pending", {"Permanent", "Failed", "Skipped", "Restored", "Purged", "Lost", "Uncertain"}, {
    "Pending": {"ok_trash": "Trashed", "ok_permanent": "Permanent", "error": "Failed", "skip": "Skipped",
                "startup_found_in_trash": "Trashed", "startup_source_present": "Failed", "startup_unknown": "Uncertain"},
    "Trashed": {"restore": "Restored", "purge": "Purged", "check_absent": "Lost"},
    "Permanent": {}, "Failed": {}, "Skipped": {}, "Restored": {}, "Purged": {}, "Lost": {}, "Uncertain": {},
})


def reachable(machine):
    start, _, table = machine
    seen, queue = {start}, deque([start])
    while queue:
        for nxt in table[queue.popleft()].values():
            if nxt not in seen:
                seen.add(nxt)
                queue.append(nxt)
    return seen


def check(name, machine, relaunchable=False):
    start, terminals, table = machine
    problems = []
    seen = reachable(machine)
    problems += [f"inatteignable : {s}" for s in table if s not in seen]
    for s, events in table.items():
        if not events and s not in terminals:
            problems.append(f"impasse non déclarée : {s}")
        if events and s in terminals and not relaunchable:
            problems.append(f"terminal avec sortie : {s}")
        for nxt in events.values():
            if nxt not in table:
                problems.append(f"transition vers un état inconnu : {s} -> {nxt}")
    print(f"{name:10} {len(table)} états, {len(seen)} atteignables, {sum(len(e) for e in table.values())} transitions : {'OK' if not problems else 'ÉCHEC'}")
    for p in problems:
        print("   -", p)
    return not problems


# Couplage élément de nettoyage x entrée d'historique : aucun fichier n'est touché sans entrée « Pending » préalable.
JOINT = {
    ("Waiting", None): {"write_pending": ("Waiting", "Pending"), "write_fail": ("Aborted", None), "skip": ("Skipped", "Skipped")},
    ("Waiting", "Pending"): {"act": ("Acting", "Pending")},
    ("Acting", "Pending"): {"ok_trash": ("Done", "Trashed"), "ok_permanent": ("Done", "Permanent"), "error": ("Error", "Failed"), "crash": ("Crashed", "Pending")},
    ("Crashed", "Pending"): {"found_in_trash": ("Crashed", "Trashed"), "source_present": ("Crashed", "Failed"), "unknown": ("Crashed", "Uncertain")},
}


def check_joint():
    seen, queue, bad = {("Waiting", None)}, deque([("Waiting", None)]), []
    while queue:
        state = queue.popleft()
        if state[0] in {"Acting", "Done", "Error", "Crashed"} and state[1] is None:
            bad.append(state)
        for nxt in JOINT.get(state, {}).values():
            if nxt not in seen:
                seen.add(nxt)
                queue.append(nxt)
    print(f"{'couplage':10} {len(seen)} états conjoints : {'OK, jamais d action sans entrée' if not bad else 'ÉCHEC ' + str(bad)}")
    return not bad


if __name__ == "__main__":
    results = [
        check("analyse", ANALYSE, relaunchable=True),
        check("nettoyage", NETTOYAGE),
        check("entrée", ENTREE),
        check_joint(),
    ]
    sys.exit(0 if all(results) else 1)
