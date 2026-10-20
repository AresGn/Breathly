# Processus « à examiner » : relevé Linux, 2026-10-20

Alimente la décision du ticket #102 (PRD D6, US-014, invariant 10 d'`AGENTS.md`).

## Question

Quelles définitions testables de « inconnu », « emplacement inhabituel », « gourmand » et « service système » donnent une section « À examiner » ni muette ni alarmiste ?

## Méthode

Deux programmes jetables (`processus-2026-10-20/linux-releve.py.txt` et `linux-releve2.py.txt`) lisent `/proc`, `/var/lib/dpkg/info/*.list` et les droits des fichiers. Ils n'affichent **que des comptes et des pourcentages** : aucun nom de processus, aucun chemin complet, aucun identifiant. Machine de développement (LMDE 6, 8 cœurs, 15,8 Go de RAM), relevés après 38 minutes d'activité, charge moyenne 0,43 : **une seule machine, au repos, d'un seul développeur**. Résultats bruts : `processus-2026-10-20/releve-*.txt`.

## Mesuré

| Mesure | Résultat |
|---|---|
| Processus vus / threads du noyau | 311 / 140 |
| Processus restants par compte | 135 du compte courant, 27 root, 9 d'autres comptes |
| Exécutable illisible (autre compte ou droits) | 38 sur 171 |
| Rattachés à un service systemd (lecture de `/proc/<pid>/cgroup`, sans programme externe) | 34 |
| Exécutable sous `/usr` / `/opt` / dossier personnel | 94 / 35 / 4 |
| Exécutable supprimé, ou dans `/tmp`, `/var/tmp`, `/dev/shm`, `memfd` | 0 |
| Exécutable protégé en écriture (propriétaire root, ni le groupe ni les autres ne peuvent écrire, dossiers parents compris) **et** fourni par un paquet dpkg | 128 sur 133 lisibles |
| Exécutable modifiable ou hors paquet | 5 sur 133 lisibles (4 hors paquet) ; 4 d'entre eux sous le dossier personnel (3 dans un dossier caché, 1 non caché) |
| RAM par processus (50 % / 90 % / 99 % / max) | 22 / 260 / 710 / 770 Mo |
| CPU par processus, 5 s (50 % / 90 % / 99 % / max, en % d'un cœur) | 0,0 / 0,6 / 7,4 à 8,0 / 10 |
| Processus au-dessus de 5 % d'un cœur | 2 à 3 |
| Logiciels (regroupés par exécutable) : 70 ; RAM 50 % / 90 % / max | 12 / 145 / 8 139 Mo |
| Logiciels au-dessus de 1 Go / 2 Go de RAM | 2 / 2 (soit 6,3 % et 12,7 % de la RAM, pour le plus gros) |
| Logiciels au-dessus de 10 % / 25 % d'un cœur | 2 / 1 |

Deux relevés à quelques minutes d'écart sont identiques à un processus près.

## Ce qu'on en tire

- Aucun processus de cette machine n'est dans un emplacement temporaire : une règle « emplacement inhabituel = dossier temporaire, exécutable supprimé, `memfd` » a **0 fausse alerte ici**. Une règle « tout ce qui n'est pas sous `/usr`, `/opt` ou `/snap` » en aurait 4 (des outils installés par l'utilisateur dans son dossier personnel) : **trop bruyante**.
- « Inconnu » défini par une **liste de noms** serait faux sur toute autre machine. « Protégé en écriture et fourni par le gestionnaire de paquets » classe 128 sur 133 ici ; il dépend du gestionnaire (dpkg lu ici ; rpm, pacman non essayés).
- « Gourmand » : au repos, le processus le plus lourd est à 10 % d'un cœur. Un seuil absolu de CPU est inopérant sur cette machine ; une RAM exprimée en part de la RAM totale s'adapte au matériel.
- Un processus d'un autre compte est illisible : il ne peut pas être jugé « inconnu » (il serait « non vérifiable », jamais « à examiner »).

## Non vérifié

- **Windows** : aucun relevé (pas de chiffres sur les services, les signatures, `%TEMP%`, `AppData\Local\Programs`). Toute définition Windows est une hypothèse.
- Autres distributions Linux, autres gestionnaires de paquets (rpm, pacman, Flatpak, snap), machine chargée, plusieurs utilisateurs.
- Taux de fausses alertes sur un poste non technique : aucune donnée.
- Un malware qui s'installe dans un emplacement protégé ou qui imite un logiciel connu : hors de portée de ces règles (Breathly n'est pas un antivirus).
