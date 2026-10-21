# Chauffe : « forte charge » et « au repos », relevé Linux, 2026-10-21

Alimente la décision du ticket #103 (PRD D4, D5, §17 point 6 ; US-010.3, US-013.1).

## Question

À partir de quelle charge du processeur, et mesurée sur quelle durée, le diagnostic dit-il « sous forte charge » ou « au repos », sachant qu'il dure moins de 10 secondes (D1) ?

## Méthode

Un programme jetable (`chauffe-2026-10-21/linux-releve-chauffe.py.txt`) lit chaque seconde la température « Package id 0 » (`coretemp`), la charge globale (`/proc/stat`, ligne `cpu`) et la charge moyenne sur 1 minute (`/proc/loadavg`), pendant trois phases : usage courant (navigateur, éditeur et bureau ouverts, 30 s), charge forte synthétique sur les 8 fils (60 s), refroidissement (120 s). Il mesure aussi le temps de processeur d'un parcours de `/proc` semblable à celui du diagnostic (ADR 0021). Il n'affiche que des températures, des pourcentages et des durées. Machine : portable HP, Intel Core i5-8265U (4 cœurs, 8 fils), LMDE 6, noyau 6.1, sur secteur. Résultat brut : `chauffe-2026-10-21/releve-chauffe-1.txt`. **Une seule machine, un seul relevé.**

## Sources lues (2026-10-21)

- proc(5), Linux man-pages 6.03 (2023-02-10) : `/proc/stat`, ligne `cpu` (temps cumulés `user`, `nice`, `system`, `idle`, `iowait`, `irq`, `softirq`, `steal` ; `iowait` « not reliable ») ; `/proc/loadavg` (« number of jobs in the run queue (state R) or waiting for disk I/O (state D) averaged over 1, 5, and 15 minutes »).
- Microsoft Learn, « GetSystemTimes function (processthreadsapi.h) », mise à jour du 2025-08-15 : temps d'inactivité, noyau et utilisateur, sommés sur tous les processeurs (64 au plus) ; « This time value also includes the amount of time the system has been idle » pour le temps noyau.

## Mesuré

| Mesure | Résultat |
|---|---|
| Charge globale par fenêtre de 5 s, usage courant | 14 à 41 % de la capacité totale (moyenne sur 1 s : 26 %) |
| Charge globale par fenêtre de 5 s, poste calme (fin du refroidissement) | 6 à 14 % |
| Charge globale, charge forte synthétique | 98 à 100 % |
| Température, usage courant / charge forte / fin du refroidissement | 54 à 70 °C / 61 à 75 °C / 49 à 51 °C |
| Retour de la température après l'arrêt de la charge forte | 75 → 61 °C en 5 s ; ≤ 56 °C en 30 s |
| Charge moyenne sur 1 min par fil, 5 s après le **début** de la charge forte | 0,51 (la charge réelle est à 100 %) |
| Charge moyenne sur 1 min par fil, 5 s puis 30 s après la **fin** de la charge forte | 0,85 puis 0,73 (la charge réelle est à 17 puis 15 %) |
| Écart entre deux lectures successives de la température à faible charge | jusqu'à 8 °C (54 → 62 → 52 °C en 10 s) |
| Coût d'un parcours de `/proc` (372 processus, `stat`, `smaps_rollup`, `cgroup`, `exe`) | 0,45 s de processeur, soit 1,1 % de la capacité totale sur 5 s |
| Température la plus haute observée | 75 °C : le seuil de 80 °C (D4) n'est jamais atteint sur cette machine |

## Ce qu'on en tire

- **La charge moyenne sur 1 min est en retard** dans les deux sens : elle sous-estime une charge qui commence et surestime une charge finie depuis 30 s, alors que la température suit la charge en quelques secondes. Elle compte aussi les tâches en attente du disque (proc(5)) et n'a pas d'équivalent direct dans `GetSystemTimes`. À écarter.
- **Une fenêtre de 5 s suffit ici** : la température du processeur suit la charge en quelques secondes. Le risque « chaud juste après une forte charge, donc jugé sans charge » est faible sur ce portable ; non vérifié sur un fixe à gros radiateur.
- **Une lecture isolée de la température est bruitée** (8 °C d'écart) : prendre la médiane de plusieurs lectures sur la fenêtre.
- **Usage courant ≠ repos** : un navigateur et un éditeur ouverts font 14 à 41 %, un poste calme 6 à 14 %. Un seuil de repos vers 20 % sépare les deux sur cette machine.
- Le parcours de `/proc` du diagnostic coûte environ 1 % de la capacité : à retirer de la mesure, mais sans effet sur les paliers.

## Non vérifié

- **Windows** : aucun relevé ; la charge globale par `GetSystemTimes` est documentée, pas essayée. La lecture de la température sans droits (#105) est hors de ce ticket.
- Le seuil de 80 °C : jamais atteint ici ; aucune machine chaude essayée.
- Portable sur batterie, fixe, processeur AMD (`k10temp`), plus de 8 fils, machine poussiéreuse.
- Les valeurs de 20 % et 50 % ailleurs que sur cette machine : renvoyées au ticket #135.
