# Dossier de commande des maquettes de Breathly

- **Statut** : prêt à soumettre à Claude Design (étape 2.7 du parcours) ; maquettes à produire
- **Date** : 2026-10-18
- **Lecteurs** : le designer (Claude Design), puis l'équipe qui code l'interface
- **Sources** (références internes, non nécessaires au designer) : `docs/PRD.md`, `docs/stories.md` (numéros US-xxx), `GLOSSARY.md`, ADR 0001 (Svelte 5), design 0001 (états) et 0002 (système)
- **Ce fichier est autonome** : le designer n'a pas accès au dépôt. Tout ce qu'il faut est ici.

## Message à coller dans Claude Design

> Voici le dossier de commande des maquettes de Breathly (fichier ci-dessous). Produis les maquettes de **tous les écrans de l'inventaire (§5)**, en **mode clair et en mode sombre**, en **français** (anglais pour l'accueil et la confirmation de nettoyage, pour vérifier la longueur des textes). Un fichier par écran, avec l'identifiant de l'écran dans le nom. Propose d'abord la **direction visuelle** (palette, typographie, formes) en une page, puis décline-la. Respecte le vocabulaire du §4 mot pour mot. Livre aussi la liste des **jetons de design** (couleurs, typographie, espacements, rayons, ombres) avec leurs noms et leurs valeurs. Signale tout point du dossier qui te semble ambigu au lieu de le deviner.

---

## 1. Le produit

**Breathly** est un outil gratuit et libre, pour Windows et Linux, qui aide **n'importe qui** à comprendre pourquoi son PC est lent et à y remédier sans risque. Slogan : *Let your PC breathe.*

Ce que fait la première version (V0.1 « Respire ») :
- **Diagnostiquer** : espace disque, mémoire (RAM), swap, processeur (CPU), chauffe, logiciels qui consomment le plus, processus à examiner.
- **Libérer de l'espace** sans danger : caches, anciennes versions, dossiers de projets recréables, classés par niveau de risque, jamais sans confirmation.
- **Garder la trace** de ce qui a été fait (historique) et **permettre d'annuler**.

**Le principe qui ne se négocie pas : Breathly propose et explique, l'utilisateur décide.** Rien n'est supprimé sans montrer la liste et obtenir une confirmation. Aucune donnée ne quitte l'ordinateur : pas de compte, pas de publicité, pas de connexion réseau.

**Ce que Breathly n'est pas** : un antivirus, un « booster de RAM », un nettoyeur de registre. L'interface ne doit jamais le laisser croire (pas de bouclier, pas de « votre PC est protégé », pas de gros bouton « Optimiser »).

## 2. Pour qui, et comment concevoir

Trois profils d'usage ; le système (Windows, Ubuntu, Mint…) n'est pas un profil.

| Profil | Qui | Niveau technique |
|---|---|---|
| **Technique** | développeurs, administrateurs | élevé |
| **Créatif** | designers, graphistes, vidéastes | moyen |
| **Bureautique et famille** | secrétaires, parents | très faible |

**Règle de conception : chaque écran doit être utilisable par le profil « bureautique et famille ».**
- **Mode simple par défaut** : mots de tous les jours, **une action principale par écran**, aucun chemin de fichier ni nom de processus brut sans clic sur « Détails ».
- **Mode avancé**, choisi dans les Réglages et mémorisé : chemins, noms de processus, identifiants de règles, détail technique.
- Le même écran existe dans les deux modes : montrer les deux sur les écrans où ils diffèrent (Diagnostic, Espace disque, Historique).
- L'objectif mesuré : **libérer de l'espace en 3 clics au plus depuis l'accueil**, sans aide.

**Ton** : rassurant, clair, jamais alarmiste, jamais culpabilisant. Direction demandée : **calme et aéré** (beaucoup d'espace, coins arrondis, couleurs douces, une seule action principale par écran). Le designer propose la palette, la typographie et les formes ; rien n'est imposé.

## 3. Contraintes à respecter

**Plateforme.** Application de bureau (Tauri v2 + Svelte 5). Moteur d'affichage du système : WebView2 sous Windows, WebKitGTK sous Linux. WebKitGTK est lent sur certaines machines : **pas d'animation lourde**, pas d'ombres floues partout, pas de flou d'arrière-plan.

**Fenêtre.** Une seule fenêtre redimensionnable, **960 × 640 par défaut, 720 × 520 au minimum**. Navigation par **barre latérale à gauche** : Accueil, Diagnostic, Espace disque, Historique, Réglages (les versions suivantes y ajoutent leurs entrées : prévoir la place, ne rien dessiner de plus que l'écran Onglets du §5).

**Thèmes.** Clair **et** sombre, dessinés tous les deux ; un troisième choix « suivre le système » dans les Réglages.

**Langues.** Français et anglais ; **aucun texte ne s'écrit sans prévoir la version anglaise**, de longueur différente (en général plus courte). Les unités : « Go », « Mo » en français ; « GB », « MB » en anglais.

**Accessibilité (WCAG 2.2 AA).** Contrastes suffisants dans les deux thèmes ; navigation complète au clavier avec focus visible ; noms lisibles par un lecteur d'écran ; texte lisible à 200 % ; **le niveau de risque ne repose jamais sur la couleur seule** (icône + libellé + couleur).

**Hors ligne.** Breathly n'ouvre aucune connexion : **pas de police, d'icône ou d'image chargée depuis Internet**. Polices et icônes sont embarquées dans l'application (indiquer leur licence).

**Légèreté.** Peu d'images, pas de vidéo, illustrations en vectoriel simple.

## 4. Vocabulaire à utiliser mot pour mot

| Dire | Ne jamais dire |
|---|---|
| **logiciel** (VS Code, OBS, Firefox…) | application, programme, appli |
| **Breathly** (l'outil) | l'app, l'appli |
| **service Breathly** (processus de fond, V0.2) ; **service système** (celui de l'OS) | « service » seul, démon, agent |
| **diagnostic** (pourquoi le PC est lent) ; **analyse du disque** | scan, audit, check-up |
| **nettoyage**, **nettoyer** | purge, optimisation, boost |
| **élément** (ce qu'on peut nettoyer) | item, entrée |
| **corbeille**, **vidage de la corbeille**, **suppression définitive** | poubelle |
| **mise en veille** d'un onglet | sieste, endormir |
| **À examiner** (processus ou service système inconnu et gourmand) | suspect, malveillant, virus, sain, infecté |
| **gros fichier**, **dossier de projet recréable** | fichier lourd, dossier de build |
| **profil d'usage** ; **profil de navigateur** | persona, « profil » seul |
| **mode guidé** (la commande à copier) ; **élévation** (droits administrateur, demandés par le système) | mode superadmin, root |

**Niveaux de risque (libellés fixes)** : **Sans risque** (se recrée seul), **À vérifier** (dossier recréable, ancien téléchargement), **Décision personnelle** (jamais sélectionnable, seulement montré). Chacun a une icône, une couleur et un libellé.

## 5. Inventaire des écrans

Chaque écran a un **identifiant stable** (H1, D2…) repris par les tickets. Les stories (US-xxx) sont celles de `docs/stories.md`.

### H. Accueil

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| H1 | Accueil | Un seul bouton principal **« Analyser mon PC »**. Phrase d'accueil. Résumé de la dernière analyse s'il existe (espace libre, libérable). **Bandeau « moins de 2 Go libres »** (rouge doux, bouton « Libérer de l'espace ») affiché seulement sous le seuil. | US-001, US-020 |
| H2 | Analyse en cours | Indicateur de progression, étape en cours en mots simples, bouton **Annuler** (rien n'est modifié). | US-001.3 |

### D. Diagnostic

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| D1 | État du PC | Cartes : **Disque** (libre / total), **Mémoire (RAM)**, **Swap**, **Processeur (CPU)**, **Chauffe** (température ou « non disponible »). **Au plus 5 recommandations** classées (titre simple, gain estimé, niveau de risque, bouton d'action ou d'explication). Listes **Top 10 des logiciels par RAM** et **par CPU**. Section **À examiner** (processus ou services système inconnus et gourmands, avec explication et renvoi vers l'antivirus du système). Mode avancé : noms de processus, chemins. | US-010 à US-014 |
| D2 | Chauffe | Explication par paliers : normal sous charge / un logiciel gourmand (nommé) / cause matérielle probable avec **liste de vérifications** (aération, poussière, mode d'alimentation, ventilateur) ; « consulter un technicien » seulement si la chauffe persiste. Aucune promesse de réparation. | US-013 |
| D3 | Réglage recommandé (Linux) | Recommandation de swap compressé (zram) : explication en mots simples, **bouton « Copier la commande »**, commande exacte dans un bloc **replié** « Voir la commande ». Breathly n'exécute rien. Commande à afficher pour Debian, Ubuntu et Mint : `sudo apt install zram-tools`. Sous Fedora, la recommandation n'apparaît pas (le swap sur zram y est activé par défaut). Le texte de la commande est celui de la distribution détectée : la maquette montre le cas Debian, Ubuntu, Mint. | US-015 |

### S. Espace disque

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| S1 | Résultat de l'analyse | Espace libérable total, réparti par niveau de risque (**Sans risque**, **À vérifier**, **Décision personnelle**), bouton principal **« Nettoyer ce qui est sans risque »**. | US-021, US-030 |
| S2 | Choisir les éléments | Liste des éléments **Sans risque** (cochés par défaut) et **À vérifier** (jamais cochés), chacun avec nom, **explication en une phrase**, taille, niveau de risque. Total sélectionné mis à jour. Dossiers de projets recréables dans À vérifier. Élément **« Vider la corbeille »** distinct, jamais coché. Boutons **Simuler** et **Nettoyer**. Mode avancé : chemin et règle. | US-021 à US-024, US-031, US-037 |
| S3 | Gros fichiers et dossiers | Liste des plus gros fichiers et dossiers du dossier personnel, **niveau Décision personnelle**, **jamais sélectionnables**, bouton **« Ouvrir le dossier »**. Aucune case à cocher. | US-022 |

### N. Nettoyage

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| N1 | Confirmation | Liste de ce qui sera supprimé, **ce qui va à la corbeille** et **ce qui est supprimé définitivement** (caches), espace concerné, boutons **Simuler**, **Retour**, **Nettoyer**. Rien n'est fait avant ce clic. | US-030.2 |
| N2 | Simulation | Rapport de ce qui se passerait, sans rien modifier, bouton pour passer au nettoyage ou revenir. | US-032 |
| N3 | Nettoyage en cours | Progression par éléments, bouton **Arrêter** (« termine l'élément en cours puis s'arrête »). | US-036.3 |
| N4 | Résumé | **Deux chiffres distincts : « envoyé à la corbeille »** et **« libéré maintenant »**. Éléments supprimés, ignorés (avec raison), en erreur (avec explication). Si le nettoyage a été arrêté : « arrêté par vous : X faits, Y non faits ». Bouton vers l'historique et, s'il reste de la place à gagner, vers le vidage de la corbeille. | US-030.4, US-036 |
| N5 | Vider la corbeille | **Confirmation forte** : nombre d'éléments et taille, avertissement « suppression définitive, impossible à annuler », liste consultable, boutons **Annuler** et **Vider la corbeille**. | US-037 |

### Y. Historique

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| Y1 | Historique | Liste : date, élément, taille, destination (corbeille / suppression définitive), état (annulable, annulé, plus annulable). Bouton **Annuler** sur les éléments annulables. Mode avancé : chemin, règle. | US-040, US-041 |
| Y2 | Annulation | Résultat d'une restauration : réussie, ou **échec expliqué** (emplacement d'origine occupé, élément plus dans la corbeille) sans rien écraser. | US-041.2 |
| Y3 | Effacer les entrées non annulables | Confirmation : nombre d'entrées, ce qui reste (annulables). | US-040.3 |

### R. Réglages

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| R1 | Général | **Langue** (français, anglais), **thème** (clair, sombre, système), **mode** (simple, avancé). | US-002 à US-004 |
| R2 | Seuils | Quatre seuils réglables, avec leur valeur par défaut : **espace libre minimal 2 Go** ; **disque occupé 90 %** ; **swap 50 %, seulement si la RAM disponible est sous 15 %** ; **température 80 °C au repos**. Pas de seuil de RAM ni de CPU en V0.1 (ils arrivent en V0.2, pour le suivi des logiciels inutilisés). Bouton « Rétablir les valeurs par défaut ». | US-020, US-010.3 |
| R3 | Exclusions | **Liste d'exclusion** : ajouter ou retirer un emplacement que Breathly ne touchera jamais ; rappel des emplacements toujours protégés (Documents, Images, Vidéos, Musique, Bureau, profils de navigateur, conteneurs chiffrés, dossiers .git). | US-035 |

### T. Onglets (esquisse pour la V0.3, à ne pas détailler)

| Id | Écran | Contenu et actions | Stories |
|---|---|---|---|
| T1 | Onglets | Esquisse : onglets éveillés et **en veille** par navigateur, RAM économisée (estimation), onglets protégés. Sert à fixer le langage visuel ; à ne pas détailler. | US-207 |

**Dialogues réutilisables** : confirmation, avertissement de suppression définitive, aide à la copie de commande, message d'erreur avec explication.

Ordre de grandeur : **environ 20 écrans**, soit 60 à 80 variantes une fois les états du §7 comptés.

## 6. Parcours à maquetter

1. **Première utilisation, 3 clics** : H1 → H2 → S1 → N1 → N4. C'est le parcours qui prouve « 3 clics au plus ».
2. **Espace critique** : H1 avec bandeau → Libérer de l'espace → S2 → N1 → N4.
3. **Choix manuel** : S2 en cochant des éléments À vérifier → Simuler (N2) → Nettoyer.
4. **Arrêt en cours de route** : N3 → Arrêter → N4 avec « arrêté par vous ».
5. **Annulation** : Y1 → Annuler un élément → Y2 (réussite et échec).
6. **Libérer vraiment** : N4 (envoyé à la corbeille ≠ libéré) → N5 Vider la corbeille.
7. **Réglage Linux** : D1 → D3 (copier la commande).
8. **Chauffe** : D1 → D2.

## 7. États à maquetter

Pour chaque écran de données : **chargement**, **vide** (rien à recommander, rien à nettoyer, historique vide), **erreur avec explication** et action possible, **mesure non disponible** (« non disponible », jamais une valeur inventée). Situations propres :
- diagnostic ou analyse **annulés** ;
- espace libre **critique** (bandeau) ;
- **élément ignoré** (fichier devenu ouvert, emplacement exclu) avec la raison ;
- nettoyage **interrompu** parce que l'historique ne peut pas être écrit (message clair, rien n'a été supprimé) ;
- **entrée incertaine** dans l'historique après un arrêt brutal de Breathly ;
- **annulation impossible** (corbeille vidée entre-temps) ;
- **aucune corbeille disponible** pour un élément.

## 8. Données d'exemple réalistes (à utiliser dans les maquettes)

Machine de référence : 666 Mo libres sur 221 Go ; swap pleine (7,5 sur 7,6 Go) avec 16 Go de RAM ; processeur à 70 °C avec 10 % de charge.
Logiciels : Firefox 3,4 Go, VS Code 3,2 Go, OBS Studio 1,8 Go, Brave 1,2 Go. Éléments : Cache de npm (4,2 Go, Sans risque), Cache de Cargo (6,1 Go, Sans risque), Cache de Firefox (1,9 Go, Sans risque), Anciens noyaux (6,6 Go, exemple de ce qui exigera plus tard les droits administrateur ; ne pas le proposer en V0.1), `node_modules` d'un projet ancien (1,3 Go, À vérifier), Installeur .deb dans Téléchargements (180 Mo, À vérifier), vidéo de 8,4 Go dans Vidéos (Décision personnelle, gros fichier). Les chiffres sont des exemples.

## 9. Textes : exemples de ton

À titre d'exemples, que le designer adapte (les textes définitifs viennent des fichiers de traduction) :
- Accueil : « Voyons pourquoi votre PC est lent. » / bouton « Analyser mon PC ».
- Bandeau : « Il reste moins de 2 Go sur votre disque. Libérons de l'espace. »
- Sans risque : « C'est le cache de npm. Il se recrée tout seul. »
- Confirmation : « Vous allez libérer 8,4 Go. Rien n'est supprimé avant votre accord. »
- À examiner : « Ce processus consomme beaucoup et nous ne le connaissons pas. Vérifiez-le avec l'antivirus de votre système. »
- Vidage : « Suppression définitive : ces fichiers ne pourront plus être récupérés. »

**Tutoiement ou vouvoiement** : non décidé ; les exemples ci-dessus vouvoient. Le designer le signale comme question (voir §11).

## 10. Livrables attendus

1. **Direction visuelle** en une page (palette claire et sombre, typographie, formes, ton des illustrations), à valider avant le reste.
2. **Un fichier par écran** (H1 à T1), nommé avec son identifiant, **en mode clair et en mode sombre** ; mode simple et mode avancé là où ils diffèrent.
3. **États** du §7 pour chaque écran concerné.
4. **Jetons de design** nommés : couleurs (fond, surface, texte, bordure, accent, trois niveaux de risque, alerte, succès), typographie (familles, tailles, graisses, interlignes), espacements, rayons, ombres.
5. **Composants** repérés : bouton principal et secondaire, carte de mesure, ligne d'élément avec case, étiquette de niveau de risque, bandeau, dialogue, barre de progression, barre latérale.
6. **Écarts** : tout point du dossier ambigu, contradictoire ou irréalisable, listé à part.
7. **Licences** des polices et icônes utilisées.

## 11. Ce que ce dossier ne décide pas

Identité visuelle (palette, typographie, logo, icône de l'application), textes définitifs, ton exact (tutoiement ou vouvoiement), animations. Ces points sont proposés par le designer puis validés par le propriétaire.

### Questions ouvertes

1. Tutoiement ou vouvoiement dans l'interface en français.
2. Logo et icône de l'application (Breathly) : à proposer.
3. Présence d'une illustration d'accueil, ou accueil purement typographique.

## 12. Réponses aux questions du designer (2026-10-18)

- **Ton** : vouvoiement.
- **Accueil** : petite illustration, vectorielle et simple.
- **Total « libérable »** : 13,7 Go, les gros fichiers étant montrés à part et jamais comptés.
- **Vider la corbeille** : case décochée dans la liste S2.
- **Commande zram** : `sudo apt install zram-tools` est la commande exacte pour Debian, Ubuntu et Mint. Elle a été vérifiée le 2026-10-18 : le paquet `zram-tools` existe dans Debian stable et dans Ubuntu 22.04, 24.04 et suivants (composant universe). Ce qui n'est pas vérifié : l'activation automatique du service après l'installation, et le cas Linux Mint lui-même. Breathly affiche la commande de la distribution détectée ; sous Fedora la recommandation n'existe pas.
- **Seuils de R2** : ceux du dossier (§5, écran R2), pas les propositions « RAM 85 %, swap 80 %, processeur 90 % ». Le seuil de swap est « 50 % avec RAM disponible sous 15 % ». RAM et CPU n'ont pas de seuil en V0.1.

## 13. Après la livraison

Les maquettes validées sont déposées dans `docs/design/maquettes/` avec un `README.md` (liste des écrans, jetons, écarts connus). Les jetons deviennent le thème de l'interface Svelte ; aucune couleur n'est écrite en dur dans un composant. Une maquette change dans la même PR que le code qui la modifie.
