# User stories de Breathly

- **Statut** : brouillon, à relire par le propriétaire (étape 2.2 du parcours)
- **Date** : 2026-10-18
- **Source** : `docs/PRD.md` (§0 à §17), `GLOSSARY.md`
- **Vocabulaire** : `GLOSSARY.md` fait foi.

## Comment lire ce fichier

- **Numéro stable.** `US-001` ne change jamais et n'est jamais réutilisé, même si la story est abandonnée (on la marque « abandonnée »). Les tickets, les specs de module et les tests citent ces numéros. V0.1 : 001 à 099 ; V0.2 : 100 à 199 ; V0.3 : 200 à 299 ; V0.4 : 300 à 399 ; V1.0 : 400 à 499.
- **Critères d'acceptation** : une phrase vérifiable, numérotée `US-030.2`, avec son cas limite. `[auto]` = test automatique ; `[manuel]` = essai sur une machine de validation (PRD §13).
- **Détail** : V0.1 est détaillée (c'est la base du premier backlog). V0.2 à V1.0 sont des stories numérotées **sans critères détaillés** : ils s'écriront quand la version approche, après les prototypes de l'étape 2.9.
- **Chiffres** : les durées et tailles du PRD §9 sont des cibles non mesurées ; les critères qui s'y rapportent sont marqués *(cible)*.
- Les noms entre parenthèses (S2, D5, E3…) renvoient aux exigences du PRD §7.
- Un critère marqué *(ajout)* est une précision que le PRD ne contient pas : à valider ou à retirer par le propriétaire.

---

# V0.1 « Respire » : diagnostic et disque

## Accueil et réglages

**US-001.** En tant qu'utilisateur, je veux ouvrir Breathly et lancer « Analyser mon PC » en un clic, afin de savoir pourquoi mon PC est lent sans rien configurer.
- US-001.1 : l'accueil affiche un seul bouton principal « Analyser mon PC » en mode simple. [manuel]
- US-001.2 : l'analyse se lance sans demander de droits administrateur ni de compte. [auto]
- US-001.3 *(ajout)* : pendant l'analyse, un indicateur de progression est visible et l'utilisateur peut l'annuler ; l'annulation ne laisse aucun fichier modifié. [auto]

**US-002.** En tant qu'utilisateur, je veux utiliser Breathly en français ou en anglais, afin de le comprendre.
- US-002.1 : la langue par défaut est celle du système si elle est prise en charge, sinon l'anglais. [auto]
- US-002.2 : changer de langue dans Réglages met à jour tous les textes sans redémarrer. [manuel]
- US-002.3 : aucune chaîne affichée n'est écrite en dur dans le code ; une clé de traduction manquante fait échouer un test. [auto]

**US-003.** En tant qu'utilisateur, je veux choisir le thème clair, sombre ou celui du système, afin de lire confortablement.
- US-003.1 : trois choix dans Réglages ; « système » suit le thème de l'OS au changement. [manuel]
- US-003.2 : le contraste des deux thèmes respecte WCAG 2.2 AA. [auto]

**US-004.** En tant qu'utilisateur, je veux passer du mode simple au mode avancé, afin de voir le détail technique seulement si je le veux.
- US-004.1 : le mode simple est actif à la première ouverture et sur chaque écran. [auto]
- US-004.2 : en mode simple, aucun chemin de fichier, nom de processus brut ni commande n'est affiché sans clic sur « Détails ». [manuel]
- US-004.3 : le choix du mode est conservé à la réouverture. [auto]

## Diagnostic

**US-010.** En tant qu'utilisateur, je veux voir l'état de mon PC (disque, RAM, swap, CPU, chauffe), afin de comprendre ce qui ralentit mon PC.
- US-010.1 : le diagnostic se termine en moins de 10 s *(cible, D1)*, analyse du disque exclue. [auto]
- US-010.2 : chaque mesure indisponible (ex. température sous Windows sans droits) affiche « non disponible », jamais une valeur inventée. [auto]
- US-010.3 : les alertes suivent les seuils de D4 (disque > 90 %, swap > 50 % avec RAM disponible < 15 %, température > 80 °C au repos) et ces seuils sont réglables. [auto]

**US-011.** En tant qu'utilisateur, je veux voir les 10 logiciels qui consomment le plus de RAM et de CPU, afin de savoir lesquels fermer.
- US-011.1 : les processus sont regroupés par logiciel (« VS Code : 3,2 Go », pas trois lignes « code »). [auto]
- US-011.2 : deux listes, par RAM et par CPU, de 10 logiciels au plus. [auto]
- US-011.3 : Breathly ne ferme aucun logiciel en V0.1 ; aucune action de fermeture n'est proposée. [auto]

**US-012.** En tant qu'utilisateur, je veux recevoir au plus 5 recommandations classées par impact, afin de savoir quoi faire en premier.
- US-012.1 : au plus 5 recommandations, chacune avec titre en langage simple, gain estimé, niveau de risque et un bouton d'action ou d'explication (D2). [auto]
- US-012.2 : s'il n'y a rien à recommander, l'écran le dit au lieu de rester vide. [auto]
- US-012.3 : une recommandation est classée au-dessus d'une autre quand son gain estimé est supérieur. [auto]

**US-013.** En tant qu'utilisateur, je veux comprendre si mon PC chauffe à cause d'un logiciel ou du matériel, afin de savoir quoi faire.
- US-013.1 : chaud sous forte charge → le texte dit que c'est normal ; chaud avec un logiciel gourmand → le logiciel est désigné ; chaud sans charge → « cause matérielle probable » et liste de vérifications (D5). [auto]
- US-013.2 *(ajout)* : « consulter un technicien » n'apparaît que si la chauffe persiste sur plusieurs mesures. [auto]
- US-013.3 : aucun texte ne promet une réparation ni ne dit qu'un composant est abîmé. [manuel]

**US-014.** En tant qu'utilisateur, je veux voir les processus et services système à examiner, afin de repérer ce qui consomme sans que je le sache.
- US-014.1 : un processus ou service système inconnu et gourmand, ou lancé depuis un emplacement inhabituel, est marqué « À examiner » avec une explication (D6). [auto]
- US-014.2 : les mots « virus », « sain », « malveillant » n'apparaissent dans aucun texte de ce marquage ; l'écran renvoie vers l'antivirus du système. [auto]
- US-014.3 : Breathly ne supprime ni ne désactive rien de ce qui est marqué « À examiner ». [auto]

**US-015.** En tant qu'utilisateur Linux, je veux savoir si le swap compressé (zram) m'aiderait, afin de corriger un swap saturé.
- US-015.1 : si le swap dépasse son seuil et que zram est absent, une recommandation l'explique et affiche la commande exacte à copier (D7, mode guidé). [auto]
- US-015.2 : Breathly n'exécute aucune commande d'installation en V0.1 ; le bouton copie la commande dans le presse-papiers. [auto]
- US-015.3 : la recommandation n'apparaît pas sur une distribution dont le réglage n'est pas vérifié (Y3). [auto]

## Espace disque

**US-020.** En tant qu'utilisateur, je veux être alerté quand il me reste moins de 2 Go, afin de nettoyer avant que le PC bloque.
- US-020.1 : à l'ouverture, un bandeau « moins de 2 Go libres » avec un bouton de nettoyage s'affiche quand l'espace libre du disque système est inférieur au seuil (2 Go par défaut, réglable) (G7). [auto]
- US-020.2 *(ajout)* : le bouton mène au nettoyage en un clic et le bandeau disparaît quand l'espace libre repasse au-dessus du seuil. [auto]
- US-020.3 : aucune notification système n'est envoyée en V0.1. [auto]

**US-021.** En tant qu'utilisateur, je veux voir ce qui occupe mon disque, classé par niveau de risque, afin de choisir quoi nettoyer.
- US-021.1 : l'analyse du disque parcourt le dossier personnel et les emplacements des règles ; ~500 000 fichiers en moins de 60 s sur un SSD *(cible, S1)*. [auto]
- US-021.2 : chaque élément est classé 🟢, 🟡 ou 🔴 et affiche la taille libérée. [auto]
- US-021.3 : un élément qui n'appartient à aucune règle n'est jamais proposé au nettoyage (S2). [auto]
- US-021.4 : l'analyse peut être annulée et ne modifie aucun fichier. [auto]

**US-022.** En tant qu'utilisateur créatif, je veux voir mes plus gros fichiers et dossiers, afin de décider moi-même de ce que je garde.
- US-022.1 : la liste affiche les plus gros fichiers et dossiers du dossier personnel, classés 🔴, jamais sélectionnés par défaut. [auto]
- US-022.2 : cette liste est informative : Breathly propose « Ouvrir le dossier » et aucune suppression ; l'utilisateur supprime avec son explorateur de fichiers (décision du 2026-10-18). [auto]
- US-022.3 : aucun élément de cette liste n'est jamais sélectionnable pour un nettoyage. [auto]

**US-023.** En tant que développeur, je veux voir les dossiers de projets recréables (`node_modules`, `target`, `.gradle`), afin de récupérer de la place sur les vieux projets.
- US-023.1 : ces dossiers sont classés 🟡 et proposés élément par élément, jamais en bloc. [auto]
- US-023.2 *(ajout)* : un dossier contenant des modifications non commitées d'un dépôt `.git` n'est jamais proposé. [auto]
- US-023.3 : un dossier n'est reconnu que par une règle explicite « nom + fichier marqueur dans le même dossier » (par exemple `node_modules` à côté de `package.json`, `target` à côté de `Cargo.toml`) ; la recherche ne descend pas dans les dossiers `.git` (décision du 2026-10-18). [auto]
- US-023.4 : un dossier au nom reconnu mais sans fichier marqueur n'est pas proposé. [auto]

**US-024.** En tant qu'utilisateur, je veux qu'on m'explique chaque élément en une phrase, afin de ne rien supprimer sans comprendre.
- US-024.1 : chaque élément affiche une explication en une phrase, dans la langue choisie (principe 4). [auto]
- US-024.2 : une règle sans explication FR et EN fait échouer la validation des règles. [auto]

## Nettoyage

**US-030.** En tant qu'utilisateur, je veux nettoyer les éléments 🟢 en un clic, afin de libérer de l'espace sans risque.
- US-030.1 : un utilisateur non technique libère de l'espace en 3 clics au plus depuis l'accueil *(cible, §9)*. [manuel]
- US-030.2 : avant de supprimer, Breathly affiche la liste de ce qui sera supprimé et l'espace libéré, et demande une confirmation (principe 1). [auto]
- US-030.3 : sans confirmation, aucun fichier n'est modifié. [auto]
- US-030.4 : en cas de succès, l'écran affiche l'espace réellement libéré. [auto]

**US-031.** En tant qu'utilisateur, je veux choisir les éléments 🟡 un par un, afin de garder la main sur ce qui demande un jugement.
- US-031.1 : aucun élément 🟡 n'est coché par défaut ; aucun élément 🔴 n'est jamais coché par défaut. [auto]
- US-031.2 : le total sélectionné se met à jour à chaque case. [manuel]

**US-032.** En tant qu'utilisateur, je veux simuler un nettoyage, afin de voir ce qui se passerait sans rien faire.
- US-032.1 : la simulation est disponible pour tout nettoyage et ne modifie aucun fichier ni l'historique réel. [auto]
- US-032.2 *(ajout)* : le résultat de la simulation est identique à ce que fera le nettoyage sur le même état du disque. [auto]

**US-033.** En tant qu'utilisateur, je veux que mes fichiers aillent à la corbeille, afin de pouvoir les récupérer.
- US-033.1 : les fichiers de l'utilisateur sont envoyés à la corbeille du système (S3). [auto]
- US-033.2 : seuls les caches 🟢 sont supprimés définitivement, et l'écran le dit avant la confirmation. [auto]
- US-033.3 : si la corbeille est indisponible pour un élément, il n'est pas supprimé ; l'erreur est expliquée. [auto]

**US-034.** En tant qu'utilisateur, je veux la garantie que Breathly ne touche jamais à mes documents, afin de lui faire confiance.
- US-034.1 : aucun nettoyage ne touche à un emplacement hors des règles, aux profils de navigateur, à `Documents`, `Images`, `Vidéos`, `Musique`, `Bureau`, aux conteneurs chiffrés (`.hc`, `.vc`, `.kdbx`), aux dossiers `.git` ni aux fichiers ouverts par un logiciel (S5). [auto, un test par garde-fou]
- US-034.2 : aucun réglage ne désactive ces garde-fous. [auto]
- US-034.3 *(ajout)* : un lien symbolique ou une jonction qui pointe vers un emplacement interdit n'est pas suivi. [auto]

**US-035.** En tant qu'utilisateur, je veux exclure des emplacements du nettoyage, afin de protéger ce que je choisis.
- US-035.1 : la liste d'exclusion prime sur toutes les règles (S4). [auto]
- US-035.2 : l'utilisateur ajoute et retire un emplacement dans Réglages. [manuel]

**US-036.** En tant qu'utilisateur, je veux qu'une erreur sur un élément n'arrête pas le reste, afin de ne pas devoir tout recommencer.
- US-036.1 : une erreur pendant un nettoyage n'interrompt pas les autres éléments ; chaque erreur est affichée avec une explication (§9). [auto]
- US-036.2 : le résumé final distingue éléments supprimés, ignorés et en erreur. [auto]

## Historique et annulation

**US-040.** En tant qu'utilisateur, je veux consulter l'historique de ce que Breathly a fait, afin de vérifier qu'il n'a touché que ce qui était prévu.
- US-040.1 : chaque suppression est inscrite : date, élément, taille, destination (corbeille ou définitive) (S6). [auto]
- US-040.2 : l'historique est un fichier local ; aucune donnée n'en sort de la machine. [auto]
- US-040.3 *(ajout)* : l'utilisateur peut effacer l'historique ; l'effacement n'annule pas les suppressions. [auto]

**US-041.** En tant qu'utilisateur, je veux annuler un nettoyage passé par la corbeille, afin de réparer une erreur.
- US-041.1 : un élément envoyé à la corbeille est restauré à son emplacement d'origine depuis l'historique. [auto]
- US-041.2 *(ajout)* : si l'emplacement d'origine est occupé ou la corbeille vidée, l'annulation échoue avec une explication, sans rien écraser. [auto]
- US-041.3 : une suppression définitive n'a pas de bouton d'annulation et l'historique l'indique. [auto]

## Contribution et qualité

**US-060.** En tant que contributeur, je veux ajouter une règle de nettoyage dans un fichier TOML sans écrire de Rust, afin d'étendre Breathly.
- US-060.1 : une règle valide (id, noms FR et EN, explications FR et EN, niveau de risque, OS, chemins) est chargée sans recompiler. [auto]
- US-060.2 : une règle invalide (explication manquante, chemin hors dossier autorisé, niveau inconnu) est refusée avec un message qui nomme la règle. [auto]
- US-060.3 : la CI valide toutes les règles du dépôt (schéma, chemins, explications FR et EN, niveau de risque) (§13). [auto]

**US-070.** En tant qu'utilisateur de clavier ou de lecteur d'écran, je veux utiliser tous les écrans sans souris, afin d'y accéder.
- US-070.1 : chaque action est atteignable et activable au clavier, avec un focus visible (WCAG 2.2 AA). [manuel]
- US-070.2 : chaque contrôle a un nom lisible par un lecteur d'écran. [auto]
- US-070.3 *(ajout)* : le texte reste lisible à 200 % de taille. [manuel]

**US-071.** En tant qu'utilisateur, je veux que Breathly n'envoie rien hors de mon PC, afin de garder mes données privées.
- US-071.1 : Breathly n'ouvre aucune connexion réseau en V0.1. [auto]
- US-071.2 : il n'y a ni compte, ni télémétrie, ni publicité. [auto]

**US-072.** En tant qu'utilisateur, je veux que Breathly soit léger, afin qu'il ne ralentisse pas mon PC.
- US-072.1 : fenêtre fermée, aucun processus de Breathly ne tourne en V0.1. [auto]
- US-072.2 : les mesures de RAM et de taille d'installeur sont relevées par le prototype et fixées par un ADR de budgets avant la première version ; la CI les contrôle quand c'est possible. [auto]

**US-073.** En tant qu'utilisateur, je veux installer Breathly facilement, afin de l'essayer.
- US-073.1 : un installeur par format est publié dans les GitHub Releases : `.msi` (Windows), `.deb`, `.rpm`, `.AppImage` (Linux). [manuel]
- US-073.2 : l'installation et la première ouverture réussissent sur les machines de validation Windows 11, Ubuntu et Linux Mint. [manuel]

---

# V0.2 « Un seul navigateur » (US-100 à US-199, critères à écrire)

- **US-100.** En tant qu'utilisateur, je veux être prévenu quand un deuxième navigateur s'ouvre, afin de ne pas en garder deux par accident (B1, B2) ; la liste des navigateurs est un fichier TOML que les contributeurs complètent (B4).
- **US-101.** Je veux choisir « Fermer A », « Fermer B », « Garder les deux » ou « Ne plus demander pendant 1 h », afin de décider sans quitter mon travail.
- **US-102.** Je veux que la fermeture d'un navigateur soit propre, afin de retrouver ma session (B3).
- **US-103.** Je veux une liste blanche de navigateurs, afin de ne pas être alerté pour ceux que j'utilise ensemble.
- **US-104.** Je veux être prévenu quand un navigateur dépasse un seuil de RAM réglable.
- **US-110.** Je veux un service Breathly avec icône dans la barre des tâches, lancé au démarrage (désactivable), afin que la surveillance tourne sans fenêtre (M1).
- **US-111.** Je veux mettre le service Breathly en pause une heure (M2).
- **US-112.** Je veux voir sur l'écran « Navigateurs » la RAM et le nombre de processus de chaque navigateur.
- **US-120.** Je veux être prévenu quand un logiciel gourmand est inutilisé depuis 15 min (réglable), afin de décider de le fermer ; Breathly ne ferme rien sans mon clic (A1, A2).
- **US-121.** Je veux une liste blanche de logiciels jamais signalés, et que les logiciels qui enregistrent, lisent ou appellent ne le soient pas (A3).
- **US-122.** Je veux une notification système quand l'espace libre passe sous 2 Go, au plus une fois par jour (A4).
- **US-123.** Je veux un écran « Logiciels ouverts » avec les gros consommateurs et l'inactivité.

# V0.3 « Sieste des onglets » (US-200 à US-299, critères à écrire)

- **US-200.** Je veux installer l'extension dans Firefox, Zen, LibreWolf, Chrome, Brave, Edge ou Chromium.
- **US-201.** Je veux que les onglets inactifs soient mis en veille en gardant position, épinglage, titre et icône (T1).
- **US-202.** Je veux que l'onglet actif, sonore, en partage d'écran, protégé ou en saisie ne soit jamais mis en veille (T2).
- **US-203.** Je veux que les onglets du quotidien restent éveillés plus longtemps, calculés sur ma machine sans sortie d'URL (T3).
- **US-204.** Je veux voir, modifier et effacer les données d'apprentissage (T4).
- **US-205.** Je veux que les onglets les moins utilisés soient mis en veille en priorité quand la RAM libre passe sous un seuil.
- **US-206.** Je veux que l'extension fonctionne seule, en mode simplifié, sans Breathly.
- **US-207.** Je veux voir l'écran « Onglets » : éveillés, en veille, protégés, RAM économisée (estimation).

# V0.4 « Démarrage léger » (US-300 à US-399, critères à écrire)

- **US-300.** Je veux lister les logiciels et services système lancés au démarrage avec leur RAM, et les désactiver et réactiver en un clic.
- **US-310.** Je veux nettoyer des éléments qui exigent les droits administrateur (anciens noyaux, snap, flatpak, journaux systemd, caches apt et dnf, Windows Update, Timeshift) par élévation ponctuelle (E1, E2).
- **US-311.** Je veux un mode guidé quand l'élévation est refusée ou indisponible (E3, E4).
- **US-320.** Je veux que Breathly applique les réglages recommandés (zram, swappiness ; mode d'alimentation et démarrage rapide sous Windows) avec liste des changements, confirmation, journal dans l'historique et retour arrière (Y1 à Y3).

# V1.0 « Stable » (US-400 à US-499, critères à écrire)

- **US-400.** Je veux des installeurs signés pour Windows et Linux, afin qu'aucun avertissement de sécurité ne m'arrête.
- **US-401.** Je veux des mises à jour automatiques, désactivables, seule connexion réseau de Breathly (§9).
- **US-402.** Je veux des extensions publiées sur addons.mozilla.org, le Chrome Web Store et Edge Add-ons.
- **US-403.** En tant que contributeur, je veux un guide de contribution des règles et des traductions et un site de documentation.
- **US-404.** Je veux qu'un audit de sécurité des actions de suppression ait été fait avant la V1.0.

---

# Questions ouvertes

1. **Format, emplacement et durée de conservation de l'historique** (US-040) : ADR « format de l'historique et des réglages ».
2. **Mises à jour avant la V1.0** : décidé le 2026-10-18, téléchargement manuel depuis les GitHub Releases, sans connexion réseau de Breathly (US-071) ; le README l'explique. Les mises à jour automatiques arrivent avec US-401.

## Décisions prises en rédigeant ces stories (2026-10-18)

- Gros fichiers : Breathly les montre et ouvre leur dossier, sans les supprimer (US-022). S2 reste absolu.
- Dossiers de projets recréables : règle explicite nom + fichier marqueur (US-023).

---

# Traçabilité

| Exigence du PRD | Stories |
|---|---|
| G1 | US-010 à US-014 |
| G2 | US-021 à US-036 |
| G3, G4 | US-100, US-120 (V0.2), US-200 à US-207 (V0.3) |
| G5 | US-072 |
| G6 | US-060 |
| G8 | US-014 |
| G9 | US-015 (V0.1), US-320 (V0.4) |
| D1 à D7 (D3 : US-011.1) | US-010 à US-015 |
| S1 à S6 | US-021, US-023, US-033 à US-035, US-040, US-041 |
| G7 (espace faible) | US-020 |
| §9 (NFR) | US-002, US-003, US-030.1, US-070 à US-073 |
| B1 à B4, M1, M2 | US-100 à US-112 |
| A1 à A4 | US-120 à US-123 |
| T1 à T4 | US-200 à US-207 |
| E1 à E5, Y1 à Y3 | US-310, US-311, US-320 (E3 : US-015 en V0.1) |
