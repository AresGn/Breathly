# ADR 0022 : « Forte charge » et « au repos » pour la chauffe

- **Statut** : accepté ; valeurs de départ **non vérifiées** hors d'une machine Linux, à confirmer dans le ticket #135
- **Date** : 2026-10-21
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le PRD D4 déclenche une alerte au-delà de 80 °C « au repos » ; D5 distingue « chaud sous forte charge » (normal), « chaud avec un processus gourmand » (cause logicielle) et « chaud sans charge » (cause matérielle probable). Aucun document ne disait à partir de quelle charge, ni sur quelle durée. Le diagnostic complet dure moins de 10 s (D1). Ticket #103. Relevé, sources et limites : `docs/research/2026-10-21-chauffe-charge-et-repos.md`.

**Mesuré** (un portable Linux, 8 fils) : usage courant (navigateur, éditeur) 14 à 41 % de la capacité totale par fenêtre de 5 s ; poste calme 6 à 14 % ; la température suit la charge en quelques secondes (75 → 61 °C 5 s après l'arrêt d'une charge forte) ; la charge moyenne sur 1 min reste en retard de plusieurs dizaines de secondes ; deux lectures de température successives peuvent différer de 8 °C ; le parcours de `/proc` du diagnostic coûte environ 1 % de la capacité. Le seuil de 80 °C n'a jamais été atteint.

**Non vérifié.** Windows ; une machine qui dépasse 80 °C ; un fixe ; un processeur AMD ; un portable sur batterie.

## Options

1. **Charge globale sur la fenêtre de 5 s du diagnostic (retenue)** : la même fenêtre que la mesure du CPU par logiciel (ADR 0021), deux seuils en part de la capacité totale.
2. Charge moyenne sur 1 minute (`/proc/loadavg`) : immédiate, sans attente, mais en retard dans les deux sens (0,51 par fil alors que la charge est à 100 % ; 0,85 alors qu'elle est retombée à 17 %), elle compte l'attente du disque (proc(5)) et n'a pas d'équivalent direct sous Windows.
3. Fenêtre longue (30 s ou plus) pour suivre l'inertie thermique : contraire à D1, et inutile ici (la température suit la charge en quelques secondes).

## Décision

**Option 1.** Le palier est calculé par une fonction pure de `diagnose` à partir de faits fournis par `platform` (`SystemInspector`, `TemperatureReader`, `ProcessInspector`), sur **une seule fenêtre de 5 s** partagée avec la mesure du CPU par logiciel (ADR 0021), dans le budget de D1.

- **Charge globale.** Part de la capacité totale (tous les fils logiques) occupée pendant la fenêtre, temps de processeur de Breathly déduit. Linux : différence de deux lectures de la ligne `cpu` de `/proc/stat`, occupé = total − `idle` − `iowait`. Windows : différence de deux appels à `GetSystemTimes`, occupé = noyau + utilisateur − inactivité (documenté, non essayé).
- **Température.** Médiane de 5 lectures, une par seconde pendant la fenêtre (une lecture isolée varie de 8 °C). Le capteur à lire est l'affaire de `TemperatureReader` (#105). Lecture impossible : « non disponible », jamais un palier (D5).
- **Au repos** : charge globale **< 20 %**. **Forte charge** : charge globale **≥ 50 %**. Valeurs **de départ, non vérifiées** hors de la machine mesurée.
- **Paliers** (énumération fermée de trois cas, seulement si la température dépasse le seuil réglable de D4, 80 °C par défaut) :
  1. charge ≥ 50 % : **sous charge**, « c'est normal » ;
  2. charge < 20 % : **sans charge**, cause matérielle probable, liste de vérifications ; c'est l'alerte « au-dessus de 80 °C au repos » de D4 ;
  3. entre les deux : si un logiciel occupe au moins 25 % d'un cœur (seuil de « gourmand » de l'ADR 0021), **logiciel gourmand**, le plus gros est désigné ; sinon (charge répartie entre de nombreux petits processus) **sous charge**.
- Les seuils de 20 % et 50 % sont des constantes nommées, marquées « non vérifiées » dans le code, comme les 3 relevés de la persistance (US-013.2).

## Conséquences

- Ticket #110 : la fonction de palier et ses tests aux bords (19,9 / 20 / 49,9 / 50 %, température au seuil et juste au-dessus, logiciel à 24,9 et 25 % d'un cœur), avec des faits fixes, jamais la vraie machine.
- Ticket #106 (`SystemInspector`) : expose les temps cumulés globaux du processeur (occupé, total) pour le calcul de la fenêtre ; ticket #105 (`TemperatureReader`) : une lecture ponctuelle, appelée 5 fois.
- Ticket #135 : confirme ou corrige 20 %, 50 % et la médiane de 5 lectures sur au moins deux autres machines, dont une qui dépasse 80 °C.
- Limite assumée : sur une machine à forte inertie thermique, un PC encore chaud juste après une forte charge peut être classé « sans charge » ; la persistance sur 3 relevés (US-013.2) évite de conseiller un technicien sur une seule mesure.

## Impact sur l'architecture

Aucun port ni crate de plus ; le graphe de dépendances ne change pas (design 0002 §3). La fenêtre de 5 s est partagée avec l'ADR 0021 : le diagnostic ne mesure pas deux fois.
