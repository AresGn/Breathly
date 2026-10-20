# ADR 0021 : Définitions de « inconnu », « emplacement inhabituel », « gourmand » et « service système » (section À examiner)

- **Statut** : accepté pour Linux (relevé sur une machine, pages de manuel et essais cités) ; Windows : hypothèses **non vérifiées**, à essayer dans le ticket #111
- **Date** : 2026-10-20
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

Le PRD D6 et US-014 demandent de marquer « à examiner » un processus ou service système « inconnu et gourmand » ou « lancé depuis un emplacement inhabituel », sans jamais dire « virus », « sain » ni « malveillant » (invariant 10). Aucun document ne définissait ces mots. Ticket #102. Relevés, sources lues et limites : `docs/research/2026-10-20-processus-a-examiner.md`.

**Mesuré** (une machine Linux de développement, 8 cœurs, 15,8 Go de RAM, trois relevés) : aucun exécutable dans un emplacement temporaire, supprimé ou en mémoire ; 128 **processus** lisibles sur 133 ont un exécutable protégé en écriture et fourni par dpkg ; les 4 processus hors paquet ont leur exécutable dans le dossier personnel ; 38 à 39 processus ont un exécutable illisible (autres comptes) ; 70 à 73 logiciels regroupés par exécutable. Le plus gros logiciel occupe **51 à 55 % de la RAM en RSS additionnée, 36 % en PSS additionnée** : additionner la RSS compte plusieurs fois la mémoire partagée. Avec la règle retenue, **3 logiciels sont gourmands, aucun d'eux n'est inconnu (3 inconnus en tout, peu gourmands) : 0 processus « à examiner »** sur cette machine.

**Vérifié par les sources et un essai** : un exécutable effacé après le lancement apparaît suffixé « (deleted) » (proc(5)), y compris quand un nouveau fichier le **remplace par renommage**, le chemin d'origine existant toujours (essai dans un dossier temporaire). Une AppImage s'exécute depuis un montage `$TMPDIR/.mount_*` ou `/tmp/.mount_*` (source du runtime AppImage).

**Non vérifié.** Tout Windows ; les autres distributions et gestionnaires de paquets ; que dpkg remplace bien les binaires par renommage (déduit, documentation de dpkg non lue) ; une machine chargée ; le taux de fausses alertes chez des personnes non techniques ; la version minimale du noyau pour `smaps_rollup` (lisible ici, noyau 6.1).

## Options

1. **Règles structurelles, sans liste de noms (retenue)** : « inconnu » = exécutable modifiable par un utilisateur standard ; « inhabituel » = temporaire, supprimé sans remplaçant ou en mémoire ; « gourmand » = part de la RAM (PSS) ou du CPU.
2. Liste de noms attendus par système : fausse sur toute autre machine, à maintenir sans fin.
3. « Inhabituel » = tout hors dossiers système : 4 fausses alertes sur la seule machine mesurée.

## Décision

**Option 1.** Un processus a une seule de quatre issues, calculée par une fonction pure du domaine `diagnose` à partir de faits fournis par `ProcessInspector` : **À examiner**, **Non vérifiable** (exécutable illisible : autre compte, droits ; jamais « à examiner », compté en mode avancé comme dans l'ADR 0020), **Ordinaire**, ou **Ignoré** (thread du noyau, reconnu par le drapeau `PF_KTHREAD`, `0x00200000`, du champ 9 de `/proc/<pid>/stat`).

- **Inconnu.** L'exécutable n'est pas protégé en écriture : il (ou un de ses dossiers parents) n'appartient pas au compte système, ou est modifiable par le groupe ou les autres. Sous Linux, un exécutable modifiable que le gestionnaire de paquets fournit (base lisible, dpkg mesuré) reste **reconnu** ; si aucune base de paquets n'est lisible, seul le test de protection s'applique. Une AppImage (montage `.mount_*` appartenant à l'utilisateur) est donc **inconnue**, jamais « inhabituelle ». Windows (hypothèse) : exécutable hors `%SystemRoot%`, `Program Files` et `Program Files (x86)`.
- **Emplacement inhabituel.** L'exécutable est dans un dossier temporaire (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/user` ; Windows : `%TEMP%`), **hors montage AppImage** (`.mount_*` directement sous le dossier temporaire) ; ou il a été **supprimé sans remplaçant** (suffixe « (deleted) » et chemin d'origine absent, ou présent mais ni protégé ni fourni par un paquet) ; ou il vit en mémoire (`/memfd:`, memfd_create(2)). Un exécutable remplacé par une mise à jour (chemin d'origine présent, protégé ou fourni par un paquet) n'est **pas** inhabituel : il reste « Ordinaire ». Le dossier de téléchargements n'en fait pas partie. Un seul de ces faits suffit, quelle que soit la consommation (D6, « ou »).
- **Gourmand.** Mesuré **par logiciel regroupé** (D3, par exécutable) : **PSS additionnée** (`/proc/<pid>/smaps_rollup`, ligne `Pss`) ≥ 10 % de la RAM totale, ou CPU ≥ 25 % d'un cœur en moyenne sur 5 secondes. Si la PSS d'un processus est illisible, sa RSS est utilisée (surestime, donc plus prudent). Valeurs **de départ**, à ajuster sur du matériel varié (non vérifié), comme les seuils de chauffe du PRD.
- **À examiner** = (inconnu **et** gourmand) **ou** emplacement inhabituel.
- **Service système.** En V0.1 : un processus rattaché à un service du système. Linux : une ligne de `/proc/<pid>/cgroup` (format `id:contrôleurs:chemin`, cgroups(7) ; cgroup v2 mesuré, v1 non vérifié) a un chemin qui contient une unité `.service` hors `user.slice` (lecture de fichier, aucun programme externe). Windows : processus de session 0 / compte de service (hypothèse, non vérifiée). **Pas de trait de plus** : `ProcessInspector` expose `is_service` et l'origine de l'exécutable ; Breathly n'énumère pas les services arrêtés. L'explication dit « service système », jamais « service » seul (GLOSSARY).
- **Textes.** L'explication dit ce qui est constaté (« tourne depuis un dossier temporaire », « ne vient pas d'un emplacement protégé et consomme beaucoup »), jamais un jugement. Aucun texte ne contient un mot de la liste « _À éviter_ » de « À examiner » dans le GLOSSARY (suspect, malveillant, virus, sain, infecté) ; un test parcourt les catalogues français et anglais, avec les équivalents anglais (suspicious, malicious, virus, clean, infected). L'écran renvoie vers l'antivirus du système et ne propose ni fermeture ni suppression (US-014.3, invariant 10).

## Conséquences

- Ticket #111 : la fonction de classement et ses tests (faits fixes, jamais le vrai `/proc`, dont un exécutable remplacé par une mise à jour et une AppImage), les deux catalogues de textes, le test des mots interdits ; un essai Windows des trois définitions avant de les déclarer vérifiées, qui inclut les programmes de mise à jour légitimes lancés depuis `%TEMP%` (Squirrel, installeurs).
- Ticket #104 (`ProcessInspector`) : le trait expose par processus l'origine de l'exécutable (protégé, modifiable, temporaire, montage AppImage, supprimé avec ou sans remplaçant, mémoire, illisible), `is_service`, la PSS (ou la RSS en repli), le drapeau de thread du noyau et, sous Linux, « fourni par un paquet ».
- **Services système presque tous « Non vérifiables » en V0.1** : sous Linux, ils tournent surtout sous `root` (27 processus root à exécutable illisible pour 34 processus de service mesurés) ; Breathly sans droits ne peut pas lire leur exécutable. La partie « services système » de D6 et US-014 est donc quasi vide en V0.1 ; le mode avancé affiche leur nombre.
- Limites assumées : un logiciel indésirable installé dans un emplacement protégé n'est pas repéré ; une installation légitime dans un dossier modifiable peut être « à examiner » si elle est gourmande : `AppData\Local\Programs` sous Windows, et sous Linux les jeux Steam (`~/.local/share/Steam`), les AppImage, les outils de `~/.cargo` ou de JetBrains Toolbox. La section reste une invitation à examiner, jamais un verdict.

## Impact sur l'architecture

Aucun port ni crate de plus ; le graphe de dépendances ne change pas (design 0002 §3). Le trait `ProcessInspector` (design 0002 §5) gagne des champs, pas une méthode d'énumération des services.
