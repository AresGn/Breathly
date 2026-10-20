# Processus « à examiner » : relevé Linux, 2026-10-20

Alimente la décision du ticket #102 (PRD D6, US-014, invariant 10 d'`AGENTS.md`).

## Question

Quelles définitions testables de « inconnu », « emplacement inhabituel », « gourmand » et « service système » donnent une section « À examiner » ni muette ni alarmiste ?

## Méthode

Trois programmes jetables (`processus-2026-10-20/linux-releve.py.txt`, `linux-releve2.py.txt`, `linux-releve3.py.txt`) lisent `/proc`, `/var/lib/dpkg/info/*.list` et les droits des fichiers. Ils n'affichent **que des comptes et des pourcentages** : aucun nom de processus, aucun chemin complet, aucun identifiant. Machine de développement (LMDE 6, noyau 6.1, 8 cœurs, 15,8 Go de RAM). Relevés 1 et 2 après 38 minutes d'activité, charge moyenne 0,43 ; relevé 3 après 59 minutes, charge moyenne 0,8 : **une seule machine, d'un seul développeur**. Résultats bruts : `processus-2026-10-20/releve*.txt`. Un essai de remplacement par renommage dans un dossier temporaire : `processus-2026-10-20/essai-renommage.txt`.

## Sources lues (2026-10-20)

- proc(5), Linux man-pages 6.03 (2023-02-10), paquet `manpages` 6.03-2 : `/proc/pid/exe` (« If the pathname has been unlinked, the symbolic link will contain the string '(deleted)' appended to the original pathname ») ; `/proc/pid/stat` champ (9) `flags` (bits `PF_*` de `include/linux/sched.h`) ; `/proc/pid/smaps` (définition de `Pss`, part proportionnelle des pages partagées) ; `/proc/pid/statm` (renvoie vers `smaps_rollup` pour des valeurs exactes).
- memfd_create(2), Linux man-pages 6.03 (2023-02-05) : un fichier en mémoire apparaît comme `/memfd:<nom> (deleted)`.
- cgroups(7), Linux man-pages 6.03 (2023-02-05) : `/proc/[pid]/cgroup`, une ligne `id:contrôleurs:chemin` par hiérarchie ; en cgroup v2, `0::chemin`.
- `include/linux/sched.h` des en-têtes du noyau 6.1 (paquet `linux-headers-6.1.0-53-common`) : `#define PF_KTHREAD 0x00200000 /* I am a kernel thread */`.
- Runtime AppImage type 2, `src/runtime/runtime.c` (branche `main` de github.com/AppImage/type2-runtime) : le point de montage est `"%s/.mount_%.*sXXXXXX"` sous `TMPDIR`, sinon `P_tmpdir` (`/tmp`), créé par `mkdtemp`. La page « Running AppImages » de docs.appimage.org ne fixe pas cet emplacement.

## Mesuré

| Mesure | Résultat |
|---|---|
| Processus vus / threads du noyau | 311 / 140 (relevés 1-2, heuristique) ; 137 par `PF_KTHREAD` (relevé 3) |
| Processus restants par compte | 135 du compte courant, 27 root, 9 d'autres comptes |
| Exécutable illisible (autre compte ou droits) | 38 sur 171 (relevé 3 : 39) |
| Rattachés à un service systemd (lecture de `/proc/<pid>/cgroup`, cgroup v2, sans programme externe) | 34 |
| Exécutable sous `/usr` / `/opt` / dossier personnel | 94 / 35 / 4 processus |
| Exécutable supprimé, ou dans `/tmp`, `/var/tmp`, `/dev/shm`, `memfd`, ou montage AppImage | 0 |
| **Processus** dont l'exécutable est protégé en écriture (propriétaire root, ni le groupe ni les autres ne peuvent écrire, dossiers parents compris) **et** fourni par un paquet dpkg | 128 sur 133 lisibles |
| Processus à exécutable modifiable ou hors paquet | 5 sur 133 lisibles (4 hors paquet) ; 4 d'entre eux sous le dossier personnel (3 dans un dossier caché, 1 non caché) |
| RAM (RSS) par processus (50 % / 90 % / 99 % / max) | 22 / 260 / 710 / 770 Mo |
| CPU par processus, 5 s (50 % / 90 % / 99 % / max, en % d'un cœur) | 0,0 / 0,6 / 7,4 à 8,0 / 10 |
| Processus au-dessus de 5 % d'un cœur | 2 à 3 |
| Logiciels (regroupés par exécutable) : 70 ; RSS additionnée 50 % / 90 % / max | 12 / 145 / 8 139 Mo (le plus gros : **51 %** de la RAM) |
| Logiciels au-dessus de 1 Go / 2 Go de RSS additionnée | 2 / 2 |
| Logiciels au-dessus de 10 % / 25 % d'un cœur | 2 / 1 (34 % d'un cœur) |
| Relevé 3 : plus gros logiciel en RSS additionnée / en PSS additionnée | 55,2 % / 36,3 % de la RAM |
| Relevé 3 : logiciels gourmands (≥ 10 % de la RAM ou ≥ 25 % d'un cœur), en RSS / en PSS | 3 / 3, dont **0 inconnu** |
| Relevé 3 : logiciels inconnus / à emplacement inhabituel / **à examiner** (règle de l'ADR 0021) | 3 / 0 / **0** |
| Essai : exécutable remplacé par renommage pendant qu'il tourne | lien suffixé « (deleted) », chemin d'origine présent |

Les relevés 1 et 2, à quelques minutes d'écart, sont identiques à un processus près.

## Ce qu'on en tire

- Aucun processus de cette machine n'est dans un emplacement temporaire : une règle « emplacement inhabituel = dossier temporaire, exécutable supprimé, `memfd` » a **0 fausse alerte ici**. Une règle « tout ce qui n'est pas sous `/usr`, `/opt` ou `/snap` » en aurait 4 (des outils installés par l'utilisateur dans son dossier personnel) : **trop bruyante**.
- **« Supprimé » seul serait bruyant ailleurs.** Un exécutable remplacé pendant qu'il tourne apparaît « (deleted) » (essai, proc(5)). Après une mise à jour, chaque logiciel non relancé serait marqué. D'où : supprimé **sans remplaçant** seulement.
- **AppImage.** Son montage est sous le dossier temporaire (source du runtime) : la règle « dossier temporaire » marquerait toute AppImage. D'où : le montage `.mount_*` est exclu de « inhabituel » et jugé par « inconnu ».
- « Inconnu » défini par une **liste de noms** serait faux sur toute autre machine. « Protégé en écriture et fourni par le gestionnaire de paquets » classe 128 processus sur 133 ici ; il dépend du gestionnaire (dpkg lu ici ; rpm, pacman non essayés).
- « Gourmand » : additionner la RSS des processus d'un logiciel compte plusieurs fois la mémoire partagée (55,2 % contre 36,3 % en PSS pour le même logiciel). La PSS (proc(5)) est la mesure retenue. Au repos, le processus le plus lourd est à 10 % d'un cœur : un seuil de CPU seul ne suffit pas, une RAM exprimée en part de la RAM totale s'adapte au matériel.
- **La règle complète** donne 0 processus « à examiner » ici : les 3 logiciels gourmands sont protégés et fournis par un paquet.
- Un processus d'un autre compte est illisible : il ne peut pas être jugé « inconnu » (il serait « non vérifiable », jamais « à examiner »). Cela touche surtout les services système, qui tournent sous `root`.

## Non vérifié

- **Windows** : aucun relevé (pas de chiffres sur les services, les signatures, `%TEMP%`, `AppData\Local\Programs`, les programmes de mise à jour lancés depuis `%TEMP%`). Toute définition Windows est une hypothèse.
- Que dpkg remplace les binaires par renommage : déduit (le comportement par renommage est essayé, la documentation de dpkg n'a pas été lue). Aucune mise à jour réelle observée.
- Format de `/proc/<pid>/cgroup` en cgroup v1 (documenté par cgroups(7), non mesuré) ; version minimale du noyau pour `smaps_rollup` (lisible ici, absente de proc(5) 6.03).
- Autres distributions Linux, autres gestionnaires de paquets (rpm, pacman, Flatpak, snap), machine chargée, plusieurs utilisateurs, jeux Steam.
- Taux de fausses alertes sur un poste non technique : aucune donnée.
- Un malware qui s'installe dans un emplacement protégé ou qui imite un logiciel connu : hors de portée de ces règles (Breathly n'est pas un antivirus).
