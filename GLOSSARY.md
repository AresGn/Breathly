# Breathly

Breathly aide n'importe qui à comprendre pourquoi son PC est lent et à y remédier sans risque. Ce glossaire fixe le vocabulaire commun au PRD, aux ADR, aux tickets, aux écrans et au code.

Règle : un terme du domaine se dit **toujours** de la même façon, dans les documents, les tickets, les écrans, les noms de modules et les tests. Un synonyme écarté par ce glossaire (« À éviter ») n'apparaît nulle part. Un terme nouveau entre ici dans la même PR que le code qui l'introduit. Les noms de code sont en anglais ; les textes affichés viennent des fichiers de traduction (FR, EN).

## Language

### Personnes

**Utilisateur**:
Personne qui se sert de Breathly sur son PC. Elle décide : Breathly propose et explique, jamais l'inverse.
_À éviter_: client, victime, « l'utilisateur lambda »

**Profil d'usage**:
Famille d'utilisateurs définie par ce qui fait ramer leur PC, avec un niveau technique : **technique** (développeurs), **créatif** (designers, graphistes, vidéastes), **bureautique et famille** (secrétaires, parents). Le système d'exploitation est un attribut, pas un profil.
_À éviter_: persona, profil seul (confusion avec le profil de navigateur), type d'utilisateur

**Machine de validation**:
PC réel sur lequel on teste avant chaque version (Linux du développeur, Windows d'un ami, Ubuntu d'une amie). Ce n'est pas un profil d'usage.
_À éviter_: persona, machine de test

**Contributeur**:
Personne qui ajoute du code, une règle, une définition de navigateur ou une traduction. **Mainteneur** : contributeur qui peut fusionner dans `main`.

### Le produit et ses parties

**Breathly**:
L'outil dans son ensemble. On dit « Breathly », pas « l'application » ni « l'app ».
_À éviter_: l'app, l'appli, l'application

**Interface**:
La fenêtre de Breathly (Tauri + Svelte). Elle ne tourne que quand elle est ouverte et ne contient aucune règle métier. Module : `app-ui`.
_À éviter_: GUI, front

**Service Breathly**:
Le processus de fond de Breathly (icône dans la barre des tâches, surveillance, notifications), V0.2. Un seul, très léger. Module : `daemon`.
_À éviter_: démon, agent, veille Breathly, « service » sans qualificatif

**Extension**:
Le module complémentaire installé **dans un navigateur** (WebExtension, TypeScript) qui met les onglets en veille. Elle fonctionne seule en mode simplifié.
_À éviter_: plugin, add-on

**Hôte de messagerie native** (`nmhost`):
Petit binaire Rust qui relie l'extension au service Breathly (native messaging).
_À éviter_: pont, bridge

**Module**:
Unité du PRD §5 qui a un identifiant stable (`platform`, `rules`, `diagnose`, `disk-scan`, `disk-clean`, `daemon`, `browser-guard`, `app-watch`, `system-tune`, `tab-sleep`, `startup`, `app-ui`) et, en Rust, son crate.

**Mode simple / mode avancé**:
Deux niveaux d'affichage de la même interface. **Simple** par défaut : mots de tous les jours, une action principale par écran. **Avancé** : détails techniques à la demande.

### Diagnostic

**Diagnostic**:
Réponse en moins d'une minute à « pourquoi mon PC est lent ? » : disque, RAM, swap, CPU, chauffe, gros consommateurs. Module : `diagnose`. Il n'inclut pas l'analyse du disque, lancée à part.
_À éviter_: scan, check-up, audit

**Analyse du disque**:
Parcours du dossier personnel et des emplacements connus qui produit les éléments à nettoyer, classés par niveau de risque. Module : `disk-scan`.
_À éviter_: scan, indexation

**Recommandation**:
Proposition du diagnostic : titre en langage simple, gain estimé, niveau de risque, bouton d'action ou d'explication. Au plus 5, classées par impact.
_À éviter_: suggestion, conseil (réservé aux explications sans action, comme pour la chauffe)

**Palier de chauffe**:
Interprétation de la température selon la charge : normal sous charge, cause logicielle, cause matérielle probable. Ne promet jamais de réparation.
_À éviter_: surchauffe pour un diagnostic (c'est un symptôme), alerte rouge

**Gros consommateur**:
Logiciel dont la RAM ou le CPU dépasse un seuil, affiché dans la liste instantanée du diagnostic.

**Logiciel inutilisé**:
Logiciel gourmand sans fenêtre au premier plan, sans son, avec un CPU faible depuis 15 minutes (réglable). Définition de départ, à prototyper (V0.2, module `app-watch`).
_À éviter_: inactif, zombie, en veille

### Ce que Breathly observe

**Logiciel**:
Tout programme tiers de l'utilisateur (VS Code, OBS, Firefox…). Ses processus sont regroupés : « VS Code : 3,2 Go ».
_À éviter_: application, programme, appli, app

**Processus**:
Instance en cours d'exécution d'un logiciel ou d'un service système. Toujours regroupé par logiciel à l'affichage.

**Service système**:
Service de l'OS ou d'un logiciel lancé en arrière-plan (systemd, services Windows). Jamais abrégé en « service ».
_À éviter_: service seul, daemon (réservé au code du service Breathly)

**À examiner**:
Étiquette d'un processus ou service système inconnu et gourmand, ou lancé depuis un emplacement inhabituel. Elle renvoie vers l'antivirus du système ; Breathly n'est pas un antivirus.
_À éviter_: suspect, malveillant, virus, sain, infecté, « à vérifier » (réservé au niveau de risque 🟡)

**Navigateur**:
Firefox, Zen, LibreWolf, Chrome, Chromium, Brave, Edge, Opera, Vivaldi… défini dans un fichier TOML. Module : `browser-guard`.

**Profil de navigateur**:
Dossier de données d'un navigateur (mots de passe, signets, sessions). Jamais touché par un nettoyage.
_À éviter_: profil seul

**Onglet**, **onglet protégé**, **onglet du quotidien**:
Onglet d'un navigateur. *Protégé* : l'utilisateur ne veut jamais qu'il soit mis en veille. *Du quotidien* : utilisé chaque jour (fréquence d'activation sur 7 jours glissants, calculée sur la machine) ; il reste éveillé plus longtemps.
_À éviter_: tab, onglet favori

**Mise en veille d'un onglet**:
L'onglet est déchargé de la mémoire mais reste à sa place, épinglé, avec son titre et son icône ; il se recharge au clic (`tabs.discard()`). Module : `tab-sleep`.
_À éviter_: sieste (nom de la version V0.3 seulement), dormir, fermer, suspendre

**Swap** et **zram**:
Swap : espace disque servant de mémoire de secours. zram : swap compressé en mémoire, réglage Linux recommandé par le diagnostic (V0.1) puis appliqué par `system-tune` (V0.4).

### Nettoyer et agir

**Nettoyage**:
Opération complète choisie par l'utilisateur : simulation, confirmation, suppression, historique. C'est le mot affiché. Module : `disk-clean`.
_À éviter_: purge, optimisation, boost

**Règle de nettoyage**:
Fichier TOML qui décrit un emplacement nettoyable : OS, niveau de risque, explication FR et EN, commande officielle éventuelle. Aucune suppression sans règle (S2).
_À éviter_: heuristique, filtre, pattern

**Règle de réglage système**:
Fichier TOML qui décrit un réglage (zram, swappiness…) : OS, distribution, paquet, effet, retour arrière. Module : `system-tune`.

**Niveau de risque**:
Classement partout identique (libellés anglais : Safe, Review, Personal decision) : 🟢 **Sans risque** (se recrée seul), 🟡 **À vérifier** (dossier recréable, ancien téléchargement), 🔴 **Décision personnelle** (jamais sélectionnable).
_À éviter_: score, gravité, priorité

**Cache**:
Données qui se recréent toutes seules. Avec le vidage confirmé de la corbeille, seul cas de suppression définitive (🟢), ce que l'écran indique.

**Élément**:
Chose précise que Breathly propose de nettoyer : un cache, un dossier de projet, un téléchargement, ou le contenu de la corbeille. Il vient toujours d'une règle de nettoyage et porte un niveau de risque et une taille.
_À éviter_: item, entrée, fichier (trop étroit)

**Gros fichier**:
Fichier ou dossier personnel très volumineux montré en 🔴 pour information, avec « Ouvrir le dossier ». Breathly ne le supprime pas et il n'est jamais sélectionnable.
_À éviter_: fichier lourd

**Dossier de projet recréable**:
Dossier d'un projet de développement qui se reconstruit (`node_modules`, `target`, `.gradle`), classé 🟡. Reconnu par une règle « nom + fichier marqueur ».
_À éviter_: dossier de build, dépendances

**Fichier marqueur**:
Fichier à côté d'un dossier qui prouve qu'il est recréable (`package.json` pour `node_modules`, `Cargo.toml` pour `target`). Sans lui, le dossier n'est pas proposé.

**Corbeille**, **vidage de la corbeille** et **suppression définitive**:
La corbeille est la destination des fichiers de l'utilisateur. Le *vidage* est un élément 🟡 distinct et confirmé qui liste et chiffre ce qu'il détruit. Les seules suppressions définitives sont les caches 🟢 et ce vidage.

**Simulation**:
Affichage de ce qui serait fait et libéré, sans rien exécuter. Toujours disponible.

**Garde-fou**:
Interdit non désactivable (S5) : jamais hors des emplacements d'une règle, jamais les profils de navigateur, `Documents`, `Images`, `Vidéos`, `Musique`, `Bureau`, les conteneurs chiffrés, les dossiers `.git`, les fichiers ouverts.
_À éviter_: sécurité, protection (trop vague), blacklist

**Liste d'exclusion**:
Emplacements que l'utilisateur interdit à Breathly ; elle prime sur toutes les règles. **Liste blanche** : logiciels ou navigateurs que Breathly ne signale jamais.
_À éviter_: blacklist, whitelist

**Historique**:
Journal local de chaque action : date, élément, taille, destination, et annulation pour ce qui est passé par la corbeille. Ce n'est pas un journal d'audit serveur.
_À éviter_: logs, journal d'audit

**Annulation**:
Restauration depuis la corbeille d'un élément nettoyé. États affichés d'une entrée de l'historique : **annulable**, **annulé**, **plus annulable** (suppression définitive, vidée, perdue, échec, ignorée) et **incertain** (arrêt brutal pendant l'action).
_À éviter_: « non annulable » dans les écrans (on dit « plus annulable »)

### Alertes et droits

**Alerte**:
État anormal constaté par le diagnostic (disque > 90 %, espace libre < 2 Go, swap, température). Un état, pas un affichage.
_À éviter_: warning, erreur

**Bandeau**:
Affichage dans l'interface, à l'ouverture, d'une alerte et du bouton qui y répond (V0.1 : « moins de 2 Go libres »).

**Notification**:
Message du système d'exploitation émis par le service Breathly (V0.2), au plus un rappel par jour pour l'espace disque.
_À éviter_: popup, push

**Élévation**:
Demande ponctuelle de droits administrateur à l'OS (UAC, polkit) pour **une** action, après explication. Breathly ne tourne jamais en administrateur et ne manipule jamais un mot de passe.
_À éviter_: mode superadmin, root, sudo (dans les textes affichés)

**Mode guidé**:
Repli quand l'élévation est refusée ou impossible : Breathly affiche la commande exacte à copier, avec l'explication, et n'exécute rien.

## Relations

- Un **utilisateur** appartient à un **profil d'usage** et utilise une **machine** ; le système d'exploitation est un attribut de la machine.
- Une **recommandation** est produite par le **diagnostic** ; elle renvoie à un **nettoyage**, à un **réglage système** ou à une explication.
- Un **nettoyage** applique des **règles de nettoyage**, passe par la **simulation**, respecte les **garde-fous**, écrit dans l'**historique**.
- Un élément ne se supprime que s'il vient d'une **règle de nettoyage** : aucune heuristique n'en crée.
- Le **service Breathly** surveille et notifie ; il ne supprime rien. Il ne ferme un **logiciel** que sur un clic de l'utilisateur.
- L'**extension** parle au **service Breathly** par l'**hôte de messagerie native** ; les URL ne sortent jamais du navigateur, seuls le domaine et des compteurs sont transmis.
- Un processus **à examiner** n'est jamais supprimé ni déclaré « virus » ni « sain » par Breathly.
- Une action qui exige l'**élévation** est une règle de la liste fermée, avec un **mode guidé** en repli.

## Exemple de dialogue

> **Développeur :** « Le service Breathly signale OBS : 1,8 Go, inutilisé depuis 15 minutes. »
> **Expert produit :** « Dis « logiciel inutilisé ». Le service Breathly notifie ; il ne ferme rien sans le clic de l'utilisateur. »
> **Développeur :** « Et ce processus inconnu à 60 % de CPU ? »
> **Expert produit :** « Il est **à examiner**, pas « suspect » : on renvoie vers l'antivirus du système et on ne supprime rien. »

## Ambiguïtés levées

- « application », « programme », « appli » désignaient un logiciel tiers **et** Breathly lui-même : on dit **logiciel** pour le premier, **Breathly** pour le second.
- « service » désignait le processus de fond de Breathly **et** un service de l'OS : on dit **service Breathly** et **service système**, jamais « service » seul.
- « profil » désignait un profil d'usage **et** un profil de navigateur : on dit toujours **profil d'usage** ou **profil de navigateur**. « Persona » est abandonné.
- « à vérifier » désignait le niveau de risque 🟡 **et** un processus inconnu : le niveau reste **À vérifier**, le processus est **À examiner**.
- « veille » : seule la **mise en veille d'un onglet** garde ce mot ; le processus de fond n'est pas une « veille ». « Sieste » reste le nom de la version V0.3, pas un terme d'interface.
- « scan », « analyse », « diagnostic » : le **diagnostic** répond à « pourquoi c'est lent ? », l'**analyse du disque** cherche ce qu'on peut nettoyer ; « scan » est abandonné.
- « nettoyage », « suppression », « libérer » : **nettoyage** est l'opération affichée à l'utilisateur ; **suppression** décrit l'acte technique (corbeille ou définitive) ; **libérer** s'emploie pour l'espace gagné.
