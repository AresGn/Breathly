# Design 0001 : machines à états de la V0.1

- **Statut** : à relire (étape 2.4 du parcours)
- **Date** : 2026-10-18
- **Source** : `docs/PRD.md` (§6, §7), `docs/stories.md`, ADR 0003 et 0005
- **Vocabulaire** : `GLOSSARY.md` fait foi.

Breathly n'a ni permission ni rôle : une transition dépend d'un **événement** (clic, fin de tâche, erreur) et d'une **condition**. Trois objets ont un cycle de vie qui compte en V0.1. Les machines du service Breathly, des alertes, de `app-watch` et des onglets s'écrivent avant leur version (voir la fin du fichier).

## 1. Analyse (diagnostic ou analyse du disque)

Deux instances indépendantes (une par type, D1). Aucune ne modifie jamais un fichier (US-021.4).

| De | Événement | Vers | Condition | Effet |
|---|---|---|---|---|
| Inactive | `start` | Running | aucune analyse du même type en cours | progression visible |
| Done, Cancelled, Failed | `start` | Running | idem | remplace l'ancien résultat |
| Running | `finish` | Done | tous les chemins traités | résultat + avertissements éventuels |
| Running | `cancel` | Cancelled | clic de l'utilisateur | aucun fichier modifié, résultat partiel jeté |
| Running | `fatal` | Failed | erreur qui empêche toute mesure | message expliqué |
| Running | `start` | *refusé* | une analyse du même type tourne | bouton désactivé (décision du 2026-10-18) |

Une erreur sur un seul chemin n'est pas `fatal` : l'analyse se termine en `Done` avec un avertissement.

## 2. Nettoyage (opération)

| De | Événement | Vers | Condition | Effet |
|---|---|---|---|---|
| Draft | `select` / `simulate` | Draft | — | la simulation ne modifie rien (US-032.1) |
| Draft | `clean` | Confirming | au moins un élément sélectionné | liste et espace affichés |
| Confirming | `back` | Draft | clic « Retour » | rien n'a été fait |
| Confirming | `confirm` | Running | confirmation explicite | historique ouvert en écriture |
| Running | `stop` | Stopping | clic « Arrêter » | l'élément en cours se termine |
| Stopping | `item_done` | Stopped | l'élément en cours est traité | résumé « arrêté par vous : X faits, Y non faits » |
| Running | `all_ok` | Completed | tous les éléments traités sans erreur | résumé avec deux chiffres (US-030.4) |
| Running | `some_errors` | CompletedWithErrors | tous traités, au moins une erreur ou un élément ignoré | erreurs expliquées (US-036) |
| Running | `history_unwritable` | Aborted | impossible d'écrire l'historique | plus aucune suppression, message |

`Completed`, `CompletedWithErrors`, `Stopped` et `Aborted` sont **terminaux** : un nouveau nettoyage est un nouvel objet, qui repart de `Draft`.

**Chaque élément** traité pendant `Running` a son propre déroulement : `Waiting` → `Acting` → `Done` | `Error` | `Skipped`. Il est `Skipped` si les garde-fous sont revérifiés au dernier moment et refusent (fichier devenu ouvert, emplacement exclu, lien symbolique vers un emplacement interdit, S5).

## 3. Entrée de l'historique (cycle de vie d'un élément nettoyé)

| De | Événement | Vers | Condition |
|---|---|---|---|
| Pending | `ok_trash` | Trashed | envoyé à la corbeille |
| Pending | `ok_permanent` | Permanent | cache 🟢 supprimé définitivement |
| Pending | `error` | Failed | l'action a échoué |
| Pending | `skip` | Skipped | garde-fou ou exclusion |
| Pending | `startup_found_in_trash` | Trashed | au démarrage suivant : retrouvé dans la corbeille |
| Pending | `startup_source_present` | Failed | au démarrage suivant : l'original est toujours là, l'action n'a pas eu lieu |
| Pending | `startup_unknown` | Uncertain | au démarrage suivant : ni l'un ni l'autre |
| Trashed | `restore` | Restored | emplacement d'origine libre et élément encore dans la corbeille (US-041) |
| Trashed | `purge` | Purged | vidage confirmé par l'utilisateur (US-037) |
| Trashed | `check_absent` | Lost | vérifié à la demande : plus dans la corbeille |

Une restauration qui échoue (emplacement occupé, corbeille vidée entre-temps) ne change pas l'état : l'élément reste `Trashed` et un message l'explique (US-041.2). `Lost` n'est détecté que **à la demande** (ouverture de l'historique, tentative d'annulation, effacement des entrées non annulables) : rien ne tourne en arrière-plan.

**Libellés affichés** (maquette Y1) : `Trashed` → « annulable » ; `Restored` → « annulé » ; `Uncertain` → « incertain » ; `Permanent`, `Failed`, `Skipped`, `Purged`, `Lost` → « plus annulable ».

**Entrées « non annulables »** (effaçables, US-040.3) : `Permanent`, `Failed`, `Skipped`, `Restored`, `Purged`, `Lost`, `Uncertain`. `Pending` et `Trashed` ne s'effacent pas. Une entrée de plus de 12 mois est supprimée dans tous les cas (US-040.4).

## 4. Invariants

1. Une analyse ne modifie aucun fichier ; `Draft`, `Confirming` et la simulation non plus.
2. **Aucun fichier n'est touché sans entrée `Pending` préalable dans l'historique** : si l'historique n'est pas inscriptible, le nettoyage passe en `Aborted` avant toute suppression.
3. Les garde-fous (S5) sont revérifiés pour **chaque élément**, au moment d'agir.
4. Aucun élément n'est à moitié supprimé : l'arrêt demandé attend la fin de l'élément en cours.
5. Seul `Trashed` peut devenir `Restored` ou `Purged` ; `Permanent` n'est jamais annulable.

## 5. Preuve : prototype jetable

`docs/design/prototypes/etats.py` (jetable, pas une base du code Rust) parcourt tous les états atteignables et vérifie qu'il n'y a ni état inatteignable, ni impasse non déclarée, ni état terminal avec sortie. Il vérifie aussi, sur le couplage « élément x entrée d'historique » (12 états conjoints), qu'on n'arrive jamais à `Acting`, `Done`, `Error` ni `Crashed` sans entrée d'historique (invariant 2).

Résultat au 2026-10-18 :

```
analyse    5 états, 5 atteignables, 7 transitions : OK
nettoyage  8 états, 8 atteignables, 10 transitions : OK
entrée     9 états, 9 atteignables, 10 transitions : OK
couplage   12 états conjoints : OK, jamais d action sans entrée
```

Trois sondes négatives (un état orphelin, une impasse sur `Trashed`, une transition `Waiting -> Acting` sans entrée) sont détectées. Ce que le prototype **ne** prouve **pas** : le comportement réel de la corbeille de l'OS ni celui d'un plantage ; ils se testent avec le code (tests d'intégration en dossier temporaire, PRD §13).

## 6. Machines à écrire avant leur version

- **V0.2** : service Breathly (arrêté, actif, en pause 1 h), alerte de deuxième navigateur (détectée, proposée, décidée, « ne plus demander » avec échéance), notification `app-watch` (observation, signalée, ignorée), notification d'espace faible (une fois par jour).
- **V0.3** : onglet (éveillé, protégé, en veille, rechargé), mode de l'extension (autonome, connectée), apprentissage des onglets du quotidien.
- **V0.4** : élévation (demandée, accordée, refusée, mode guidé), réglage système (proposé, appliqué, annulé).
