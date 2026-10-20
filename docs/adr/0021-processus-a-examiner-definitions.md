# ADR 0021 : Définitions de « inconnu », « emplacement inhabituel », « gourmand » et « service système » (section À examiner)

- **Statut** : accepté pour Linux (relevé sur une machine, au repos) ; Windows : hypothèses **non vérifiées**, à essayer dans le ticket #111
- **Date** : 2026-10-20
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le PRD D6 et US-014 demandent de marquer « à examiner » un processus ou service système « inconnu et gourmand » ou « lancé depuis un emplacement inhabituel », sans jamais dire « virus », « sain » ni « malveillant » (invariant 10). Aucun document ne définissait ces mots. Ticket #102. Relevé et limites : `docs/research/2026-10-20-processus-a-examiner.md`.

**Mesuré** (une machine Linux de développement, 311 processus, au repos) : aucun exécutable dans un emplacement temporaire ; 128 exécutables lisibles sur 133 sont à la fois protégés en écriture et fournis par dpkg ; les 4 « hors paquet » sont des outils installés par l'utilisateur dans son dossier personnel ; 38 exécutables sont illisibles (autres comptes) ; le plus gros logiciel regroupé prend 12,7 % de la RAM et un seul dépasse 25 % d'un cœur.

**Non vérifié.** Tout Windows ; les autres distributions et gestionnaires de paquets ; une machine chargée ; le taux de fausses alertes chez des personnes non techniques.

## Options

1. **Règles structurelles, sans liste de noms (retenue)** : « inconnu » = exécutable modifiable par un utilisateur standard ; « inhabituel » = temporaire, supprimé ou en mémoire ; « gourmand » = part de la RAM ou du CPU.
2. Liste de noms attendus par système : fausse sur toute autre machine, à maintenir sans fin.
3. « Inhabituel » = tout hors dossiers système : 4 fausses alertes sur la seule machine mesurée.

## Décision

**Option 1.** Un processus a une seule de quatre issues, calculée par une fonction pure du domaine `diagnose` à partir de faits fournis par `ProcessInspector` : **À examiner**, **Non vérifiable** (exécutable illisible : autre compte, droits ; jamais « à examiner », compté en mode avancé comme dans l'ADR 0020), **Ordinaire**, ou **Ignoré** (thread du noyau).

- **Inconnu.** L'exécutable n'est pas protégé en écriture : il (ou un de ses dossiers parents) n'appartient pas au compte système, ou est modifiable par le groupe ou les autres. Sous Linux, un exécutable modifiable que le gestionnaire de paquets fournit (base lisible, dpkg mesuré) reste **reconnu** ; si aucune base de paquets n'est lisible, ce test n'est pas appliqué. Windows (hypothèse) : exécutable hors `%SystemRoot%`, `Program Files` et `Program Files (x86)`.
- **Emplacement inhabituel.** L'exécutable est dans un dossier temporaire (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/user` ; Windows : `%TEMP%`), a été **supprimé** après le lancement, ou vit en mémoire (`memfd`). Le dossier de téléchargements n'en fait pas partie. Un seul de ces faits suffit, quelle que soit la consommation (D6, « ou »).
- **Gourmand.** Mesuré **par logiciel regroupé** (D3, par exécutable) : RAM résidente ≥ 10 % de la RAM totale, ou CPU ≥ 25 % d'un cœur en moyenne sur 5 secondes. Valeurs **de départ**, à ajuster sur du matériel varié (non vérifié), comme les seuils de chauffe du PRD.
- **À examiner** = (inconnu **et** gourmand) **ou** emplacement inhabituel.
- **Service système.** En V0.1 : un processus rattaché à un service du système. Linux : `/proc/<pid>/cgroup` contient une unité `.service` hors `user.slice` (lecture de fichier, aucun programme externe). Windows : processus de session 0 / compte de service (hypothèse, non vérifiée). **Pas de trait de plus** : `ProcessInspector` expose `is_service` et l'origine de l'exécutable ; Breathly n'énumère pas les services arrêtés. Le marquage dit « service » dans l'explication.
- **Textes.** L'explication dit ce qui est constaté (« tourne depuis un dossier temporaire », « ne vient pas d'un emplacement protégé et consomme beaucoup »), jamais un jugement. Aucun texte ne contient « virus », « sain » ni « malveillant » ; un test parcourt les catalogues français et anglais. L'écran renvoie vers l'antivirus du système et ne propose ni fermeture ni suppression (US-014.3, invariant 10).

## Conséquences

- Ticket #111 : la fonction de classement et ses tests (faits fixes, jamais le vrai `/proc`), les deux catalogues de textes, le test des mots interdits ; un essai Windows des trois définitions avant de les déclarer vérifiées.
- Ticket #104 (`ProcessInspector`) : le trait expose par processus l'origine de l'exécutable (protégé, modifiable, temporaire, supprimé, mémoire, illisible), `is_service` et, sous Linux, « fourni par un paquet ».
- Limite assumée : un logiciel indésirable installé dans un emplacement protégé, ou une installation légitime dans un dossier modifiable (ex. `AppData\Local\Programs` sous Windows) peut être mal classé ; la section reste une invitation à examiner, jamais un verdict.

## Impact sur l'architecture

Aucun port ni crate de plus ; le graphe de dépendances ne change pas (design 0002 §3). Le trait `ProcessInspector` (design 0002 §5) gagne des champs, pas une méthode d'énumération des services.
