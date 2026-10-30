# Breathly — Product Requirements Document

> *Let your PC breathe.*
> Statut : **brouillon v0.2 — cadrage validé les 2026-10-17 et 2026-10-18 (§0, §17) ; user stories dans `docs/stories.md` (brouillon), specs par module encore à écrire** · Dernière mise à jour : 2026-10-18

---

## 0. Hypothèses (validées le 2026-10-17)

Ces points orientent tout le reste du document. Ils sont validés par le propriétaire ✅ ; tout changement passe par une PR qui le dit.

1. **Plateformes V1 : Windows 10/11 et Linux** (Debian, Ubuntu, Mint, Fedora). macOS est **hors V1.0** : après la V1.0 (§6). ✅
2. **Pile technique : Rust + Tauri v2** pour Breathly, **WebExtension** (TypeScript) pour l'extension navigateur. ✅
3. **Langues de l'interface : français et anglais** dès la première version ; les autres langues viendront des contributeurs. ✅
4. **Aucune télémétrie, aucun compte, aucun serveur.** Tout reste sur la machine. ✅
5. **Licence : MIT** (fichier `LICENSE` du dépôt). ✅ Décidé.
6. **Breathly ne supprime jamais rien sans montrer la liste et obtenir une confirmation.** ✅

### Décisions du 2026-10-18 ✅

7. **Droits administrateur :** élévation **ponctuelle**, une action à la fois, par la fenêtre standard de l'OS (UAC, polkit), avec explication préalable ; **mode guidé** (commande exacte à copier) quand l'élévation est impossible ou refusée. Pas de « mode superadmin » : Breathly ne tourne jamais en administrateur (§7.8).
8. **Surchauffe :** paliers et conseils guidés ; Breathly distingue cause logicielle et cause matérielle, ne promet aucune réparation et oriente vers un technicien seulement si cela persiste (D5).
9. **Utilisateurs :** trois profils d'usage (technique, créatif, bureautique et famille) ; le système d'exploitation est un attribut, pas un profil (§2). Écrans conçus pour tous, centrés sur l'utilisateur.
10. **Logiciels inutilisés :** liste instantanée des gros consommateurs de RAM **et de CPU** en V0.1 ; suivi d'inactivité (15 min, réglable) avec notification en V0.2 (`app-watch`).
11. **Espace disque faible (< 2 Go libres) :** bandeau à l'ouverture en V0.1 ; notification système avec le service en V0.2.
12. **Processus et services inconnus ou gourmands :** signalés « à examiner » sans prétendre détecter un virus ; Breathly n'est pas un antivirus (§3).
13. **Réglages du système (zram, swappiness…) :** diagnostic et commande exacte en V0.1 ; application par Breathly en V0.4 (`system-tune`), après vérification de chaque paquet et de chaque distribution.
14. **Légèreté (G5) :** budgets chiffrés **mesurés par le prototype** (étape 2.9 du parcours) et fixés par ADR ; le code est conçu dès le départ pour les tenir. Décidé le 2026-10-18 (ADR 0008) : Tauri + Svelte conservé, fenêtre ≤ 350 Mo de PSS re-mesurée par version, pas d'AppImage en V0.1.
15. **Mises à jour avant la V1.0 :** téléchargement manuel depuis les GitHub Releases, sans connexion réseau de Breathly (US-071) ; mises à jour automatiques avec la V1.0 (US-401).
16. **Gros fichiers 🔴 :** montrés avec « Ouvrir le dossier », jamais sélectionnables ni supprimés par Breathly (US-022).
17. **Corbeille :** les éléments 🟡 vont à la corbeille ; vider la corbeille est un élément 🟡 distinct et confirmé (US-037) ; deux chiffres affichés, envoyé et libéré.
18. **Règles :** intégrées au binaire à la compilation en V0.1, champ `command` informatif (US-060).
19. **Précisions de comportement validées** (détail dans `docs/stories.md`) : annulation du diagnostic, langue et thème « système », mode mémorisé, tri des recommandations, bandeau qui disparaît, simulation fidèle, liens symboliques jamais suivis, annulation qui n'écrase rien, Windows 10 et Fedora publiés « non testé ».
20. **Historique :** conservé 12 mois puis rotation ; l'utilisateur n'efface que les entrées non annulables (S6, US-040).

---

## 1. Problème

Les PC « qui rament » ont presque toujours les mêmes causes, sur tous les systèmes :

| Cause | Ce que vit l'utilisateur | Exemple réel (machine de référence, 2026-10-17) |
|---|---|---|
| **Disque plein** | Lenteurs, mises à jour qui échouent, « je ne sais pas quoi supprimer » | 666 Mo libres sur 221 Go. **~30 Go** de caches et d'anciennes versions supprimables sans risque, 6,6 Go d'anciens noyaux |
| **RAM saturée → swap** | Gels, PC lent « presque tout le temps » | Swap pleine (7,5/7,6 Go) avec 16 Go de RAM |
| **Trop d'onglets / de navigateurs** | On n'ose pas fermer ses onglets, surtout les épinglés | Firefox + Zen + Brave ouverts en même temps, dizaines d'onglets épinglés pour ~10 vraiment utilisés |
| **Logiciels lancés au démarrage** | Le PC est lent dès l'allumage | Apache, MariaDB, PostgreSQL, Redis, Tor, BreakTimer (580 Mo) lancés en permanence |
| **Logiciels ouverts mais inutilisés** | RAM et CPU occupés par des logiciels qu'on n'utilise plus (agents en arrière-plan, OBS Studio ouvert depuis des heures…) | Plusieurs agents et outils gourmands lancés et oubliés |
| **Processus et services inconnus** | Ressources consommées par un service système dont on ignore l'existence (parfois un logiciel indésirable sous Windows) | Services en arrière-plan inattendus repérés sur la machine de référence |
| **Surchauffe** | Le processeur ralentit pour se protéger. Cause logicielle (processus gourmand) ou matérielle (poussière, ventilation, pâte thermique) | 70 °C à 10 % de charge |

Les outils existants traitent chacun **un morceau** du problème et ne savent pas **expliquer** :

| Outil | Limite |
|---|---|
| BleachBit | Vide des caches, mais n'explique pas ce qui ralentit le PC et ne trouve pas les gros fichiers |
| WizTree, WinDirStat, dust, gdu | Montrent ce qui occupe le disque, sans dire ce qu'on peut supprimer |
| Czkawka, Kondo | Excellents mais spécialisés (doublons, dossiers de projets) |
| Économiseurs de mémoire des navigateurs | Ne connaissent pas les habitudes de l'utilisateur, réglages cachés, un seul navigateur à la fois |
| **Surveillance de plusieurs navigateurs ouverts en même temps** | **N'existe pas** |

**La valeur de Breathly : diagnostiquer, expliquer simplement, proposer des actions classées par niveau de risque, et ne rien faire sans accord.**

---

## 2. Utilisateurs cibles

Toute personne dont le PC rame, est souvent plein et qui ne comprend pas ce qui se passe. Le système d'exploitation ne décrit pas la personne : un développeur utilise Windows aussi bien qu'Ubuntu. On distingue donc **trois profils d'usage**, chacun avec un niveau technique ; Windows, Ubuntu, Mint ou Fedora sont des attributs.

| Profil | Qui | Ce qui le fait ramer | Besoin principal | Niveau technique |
|---|---|---|---|---|
| **Technique** | Développeurs, administrateurs, étudiants en informatique (Windows ou Linux) | Dizaines d'onglets épinglés, services de développement au démarrage, caches (`node_modules`, Gradle, Cargo), conteneurs | Garder ses onglets sans que le PC s'effondre ; nettoyer les caches ; comprendre un service qui tourne | Élevé |
| **Créatif** | Designers, graphistes, vidéastes, photographes | Gros fichiers et projets, caches de logiciels de montage et de dessin, plusieurs logiciels lourds ouverts à la fois | Retrouver de l'espace sans toucher aux projets ; savoir quel logiciel pèse sur la RAM et le CPU | Moyen |
| **Bureautique et famille** | Secrétaires, parents, personnes sans formation technique | Disque plein, démarrage lent, navigateur chargé, logiciels inconnus en arrière-plan | « Cliquer sur un bouton et comprendre ce qui se passe » | Très faible |

**Machines de validation** (tests réels avant chaque version, §13) : le Linux du développeur (LMDE), le Windows 11 d'un ami, l'Ubuntu d'une amie (Firefox en snap). Ce sont des machines de test, pas des profils d'usage.

**Règle de conception : des écrans pour tout le monde, centrés sur l'utilisateur.** Chaque écran est utilisable par le profil « bureautique et famille » : mode simple par défaut, une action principale par écran, mots de tous les jours. Le détail technique est disponible à la demande (mode avancé), jamais imposé. L'UI/UX se valide avec une personne de chaque profil.

---

## 3. Objectifs et non-objectifs

### Objectifs
- **G1.** Répondre en moins d'une minute à « pourquoi mon PC est lent ? », en langage simple (disque, mémoire, processeur, chauffe).
- **G2.** Libérer de l'espace disque **sans risque pour les données personnelles**.
- **G3.** Réduire la RAM et le CPU consommés par ce qui est ouvert sans être utilisé : **les navigateurs** (sans obliger à fermer des onglets) **et tous les autres logiciels** (un logiciel gourmand inutilisé depuis 15 min est signalé ; l'utilisateur décide).
- **G4.** Éviter d'avoir plusieurs navigateurs ouverts en même temps sans le vouloir.
- **G5.** Être **lui-même léger** : un outil anti-lenteur qui ralentit le PC n'a pas de sens. **L'architecture et le code sont conçus dès le départ pour tenir des budgets mesurables** (§9) : rien ne tourne quand ce n'est pas nécessaire, une seule instance de surveillance, des mesures échantillonnées et non continues.
- **G7.** **Alerter quand l'espace libre passe sous 2 Go** (seuil réglable), avec un nettoyage clair en un clic.
- **G8.** **Repérer les processus et services inconnus ou gourmands** et les signaler « à examiner », sans prétendre détecter un virus.
- **G9.** **Guider les réglages du système** qui aident la machine (swap compressé, mode d'alimentation, démarrage), puis les appliquer avec accord quand l'OS le permet.
- **G6.** Être **facile à étendre** par la communauté (nouveaux caches, nouveaux navigateurs, nouvelles langues) sans écrire de Rust.

### Non-objectifs (ce que Breathly ne fera jamais)
- ❌ **« Nettoyeur de registre » Windows** : gain nul, risque réel.
- ❌ **« Booster de RAM »** qui vide la mémoire de force : contre-productif.
- ❌ **Antivirus** : Breathly signale des processus « à examiner » mais ne prétend jamais détecter un virus, ne dit jamais « votre PC est sain » et ne supprime pas ces fichiers lui-même. Mise à jour de pilotes, VPN et tout ce qui sort du sujet sont aussi exclus.
- ❌ Comptes, synchronisation dans le cloud, publicité, télémétrie activée par défaut.
- ❌ Supprimer des **données personnelles** (documents, photos, profils de navigateur, conteneurs chiffrés, bases de données) **de sa propre initiative**. L'utilisateur peut les voir dans l'analyse, mais Breathly ne les propose jamais au nettoyage automatique.
- ❌ Application mobile.

---

## 4. Principes produit (validés le 2026-10-18)

1. **Montrer avant d'agir.** Toute action affiche ce qui va se passer et ce que ça libère. Une simulation est toujours possible.
2. **Réversible par défaut.** Les fichiers de l'utilisateur vont **dans la corbeille** du système. Seuls les caches (qui se recréent tout seuls) sont supprimés définitivement. Chaque action est notée dans un **historique** et peut être annulée quand c'est possible.
3. **Trois niveaux de risque, partout :**
   - 🟢 **Sans risque** : caches, fichiers temporaires, anciennes versions. Ils se recréent tout seuls.
   - 🟡 **À vérifier** : dossiers recréables (`node_modules`, installeurs déjà utilisés), anciens téléchargements.
   - 🔴 **Décision personnelle** : gros fichiers personnels. Breathly les montre, mais ne les propose jamais au nettoyage : ils ne sont pas sélectionnables (US-022).
4. **Expliquer chaque proposition en une phrase** (« C'est le cache de npm : il se recrée automatiquement au prochain `npm install` »). Le détail technique reste accessible.
5. **Ne rien casser.** En cas de doute, Breathly ne propose pas l'action. Il vaut mieux libérer 20 Go sans risque que 25 Go dont 5 de trop.
6. **Sobriété.** Le service Breathly consomme moins qu'un onglet ouvert. La fenêtre ne tourne que quand elle est ouverte.
7. **Pour tout le monde.** Mode simple par défaut, jargon absent de l'écran principal, détail technique à la demande. Breathly propose et explique ; c'est l'utilisateur qui décide (aucune fermeture de logiciel, aucune suppression ni aucun réglage sans son accord).

---

## 5. Carte des modules

Chaque module a un identifiant stable. Ces identifiants nomment les specs détaillées (`docs/specs/SPEC-<id>.md`) et les tâches.

| Module id | Responsabilité | Dépend de |
|---|---|---|
| `platform` | Abstraction du système d'exploitation : processus, RAM, swap, disques, températures, chemins standard, corbeille, droits administrateur | — |
| `rules` | Format et chargement des **règles de nettoyage** déclaratives (TOML) : emplacement, OS, niveau de risque, explication, commande éventuelle | — |
| `diagnose` | Diagnostic « pourquoi mon PC est lent » : disque, RAM, swap, CPU, chauffe, gros consommateurs, processus à examiner, recommandations | `platform` |
| `disk-scan` | Parcours rapide du disque, regroupement par catégorie, application des règles | `platform`, `rules` |
| `disk-clean` | Exécution des nettoyages : simulation, corbeille, historique, annulation | `platform`, `rules`, `disk-scan` |
| `daemon` | Service Breathly, processus de fond léger : icône dans la barre des tâches, IPC avec l'interface, planification | `platform` |
| `app-watch` | Suivi des logiciels gourmands et inutilisés (RAM et CPU), notification « fermer / ignorer », alerte d'espace disque faible | `platform`, `daemon` |
| `system-tune` | Réglages du système guidés puis appliqués (zram, swappiness, mode d'alimentation…) avec élévation ponctuelle et retour arrière | `platform`, `rules` |
| `browser-guard` | Détection des navigateurs ouverts, RAM par navigateur, alertes « un autre navigateur est déjà ouvert » | `platform`, `daemon` |
| `tab-sleep` | Extension navigateur et hôte *native messaging* : mise en veille des onglets, apprentissage des onglets du quotidien, statistiques | `daemon`, `browser-guard` |
| `startup` | Liste et désactivation des logiciels et services système lancés au démarrage | `platform` |
| `app-ui` | Interface Tauri : écrans, i18n, accessibilité, thème clair/sombre | tous les modules ci-dessus (via des commandes Tauri) |

**Ordre de construction :**
`platform`, `rules` → `diagnose`, `disk-scan` → `disk-clean` → `app-ui` (V0.1) → `daemon` → `browser-guard`, `app-watch` (V0.2) → `tab-sleep` (V0.3) → `startup`, `system-tune` (V0.4)

```
platform ─┬─> diagnose ───────────────────────┐
          ├─> disk-scan ─> disk-clean ────────┤
rules ────┘                                   ├─> app-ui
          ├─> daemon ─┬─> browser-guard ─> tab-sleep ┤
          │           └─> app-watch ───────────────┤
          ├─> startup ─────────────────────────────┤
rules ────┴─> system-tune ─────────────────────────┘
```

---

## 6. Périmètre par version

### V0.1 « Respire » : diagnostic et disque
*Une personne de n'importe quel profil, sous Windows ou Linux, peut libérer de l'espace et comprendre pourquoi son PC est lent. Validation : l'ami sous Windows et l'amie sous Ubuntu.*

**Plateformes :** Windows 10/11, Linux (Debian/Ubuntu/Mint, Fedora).

| Module | Inclus | Exclu (plus tard) |
|---|---|---|
| `platform` | Processus, RAM, swap, **charge CPU**, disques, température du processeur quand elle est disponible, corbeille, chemins standard | Élévation des droits administrateur (V0.4) ; en V0.1 le mode guidé affiche la commande à copier |
| `rules` | Format TOML, ~40 règles de départ (§8) | Règles qui demandent des droits administrateur |
| `diagnose` | Écran « État du PC » : disque, RAM, swap, CPU, chauffe interprétée par paliers (D5), **top 10 des logiciels par RAM et par CPU (processus regroupés par logiciel)**, processus et services système « à examiner » (D6), état du swap et recommandation zram avec la commande exacte (D7), recommandations en phrases simples | Historique dans le temps, suivi d'inactivité (V0.2), application des réglages (V0.4) |
| `disk-scan` | Analyse du dossier personnel + caches connus, catégories 🟢🟡🔴, **plus gros fichiers et dossiers**, dossiers de projets recréables (`node_modules`, `target`, `.gradle`…) | Doublons (on renverra vers Czkawka), analyse des disques externes |
| `disk-clean` | Nettoyage 🟢 en un clic, 🟡 élément par élément, simulation, corbeille, historique, annulation | Nettoyages système qui demandent des droits administrateur |
| `app-ui` | Accueil (« Analyser mon PC ») avec **bandeau « moins de 2 Go libres » et bouton de nettoyage**, écran Diagnostic, écran Espace disque, écran Historique, Réglages, mode simple / avancé, FR/EN, thème clair/sombre | Icône dans la barre des tâches, notifications système (V0.2) |

**Livrables :** installeur `.msi` (Windows), `.deb` et `.rpm` (Linux), publiés dans les *GitHub Releases* avec leurs empreintes SHA-256. Pas d'`.AppImage` en V0.1 (ADR 0008).

### V0.2 « Un seul navigateur » : navigateurs, logiciels inutilisés et espace disque
*Breathly prévient quand un deuxième navigateur s'ouvre, signale les logiciels gourmands oubliés et alerte quand l'espace disque devient critique.*

| Module | Inclus |
|---|---|
| `daemon` | Service Breathly + icône dans la barre des tâches, lancement automatique au démarrage (désactivable), communication avec l'interface |
| `browser-guard` | Détection de Firefox, Zen, LibreWolf, Chrome, Chromium, Brave, Edge, Opera, Vivaldi (liste extensible en TOML). Quand un **deuxième** navigateur s'ouvre, une petite fenêtre Breathly propose : **« Fermer [A] »**, **« Fermer [B] »**, **« Garder les deux »**, **« Ne plus demander pendant 1 h »**. Liste blanche. Alerte quand un navigateur dépasse X Go (seuil réglable) |
| `app-watch` | Suivi des logiciels gourmands en RAM ou CPU et **inutilisés depuis 15 min** (réglable) : notification « Fermer », « Garder », « Ne plus demander » ; liste blanche. **Notification système « moins de 2 Go libres »** (seuil réglable, un rappel par jour au plus) |
| `app-ui` | Écran « Navigateurs » : RAM par navigateur, nombre de processus, réglages des alertes ; écran « Logiciels ouverts » : gros consommateurs et inactivité |

**Contraintes connues :**
- La notion d'« inutilisé » n'est pas fournie par l'OS de façon uniforme (Wayland, Windows). Définition de départ : logiciel sans fenêtre au premier plan, sans son, avec un CPU faible pendant 15 min. **À prototyper (étape 2.9) avant de s'y engager.** Breathly ne ferme jamais un logiciel de lui-même : il notifie, l'utilisateur choisit, la fermeture est propre (B3).
- Breathly ne peut pas *empêcher* un navigateur de démarrer sans aller très loin dans le système (pilote, hook). Il **détecte** le lancement (vérification toutes les 1 à 2 s, ou événements du système quand ils existent) et **réagit** dans les 2 secondes.

### V0.3 « Sieste des onglets » : mise en veille des onglets
*Garder tous ses onglets, y compris épinglés, sans que le PC rame.*

| Module | Inclus |
|---|---|
| `tab-sleep` (extension) | Une extension pour **Firefox / Zen / LibreWolf** et **Chrome / Brave / Edge / Chromium** (même code, deux paquets). Met en veille les onglets inactifs avec l'API `tabs.discard()` : l'onglet **reste à sa place, épinglé, avec son titre et son icône**, et se recharge au clic |
| `tab-sleep` (règles) | **Jamais en veille :** l'onglet affiché, ceux qui jouent du son ou une visio, ceux protégés par l'utilisateur. **Onglets du quotidien :** Breathly apprend les onglets utilisés chaque jour (fréquence d'activation sur 7 jours) et les garde éveillés plus longtemps. **Les autres :** en veille après X minutes d'inactivité (30 par défaut). **Pression mémoire :** si la RAM libre passe sous un seuil, Breathly met en veille en priorité les onglets les moins utilisés |
| `tab-sleep` (hôte) | Hôte *native messaging* (petit binaire Rust) qui relie l'extension au service Breathly : la RAM du système est transmise à l'extension, et les statistiques à l'interface |
| `app-ui` | Écran « Onglets » : onglets éveillés et en veille par navigateur, **RAM économisée (estimation)**, onglets protégés, réglages |

**Contraintes connues :**
- Les navigateurs **ne donnent pas la RAM onglet par onglet** aux extensions. Breathly mesure la RAM exacte **par navigateur** (au niveau du système) et **estime** les économies par onglet.
- L'extension fonctionne **seule** (sans Breathly) en mode simplifié, avec des règles par défaut et sans pression mémoire.

### V0.4 « Démarrage léger » : démarrage et actions système

| Module | Inclus |
|---|---|
| `startup` | Liste des logiciels et services système lancés au démarrage, avec leur RAM ; désactivation et réactivation en un clic. Linux : autostart XDG, services systemd utilisateur et système. Windows : clés `Run`, dossier Démarrage, tâches planifiées |
| `platform` / `rules` | **Actions qui demandent des droits administrateur**, avec explication et demande de mot de passe : anciens noyaux (Debian/Ubuntu), révisions snap désactivées, flatpak inutilisés, journaux systemd, cache apt/dnf, cache Windows Update, instantanés Timeshift |
| `system-tune` | Application des réglages recommandés : Linux (zram, swappiness) via le gestionnaire de paquets de la distribution (apt, dnf), Windows (mode d'alimentation, démarrage rapide). Élévation ponctuelle (§7.8), liste des changements, retour arrière. **Chaque paquet et chaque distribution sont vérifiés dans la documentation avant écriture de la règle (non vérifié à ce jour)** |
| `diagnose` | Recommandations Linux : zram, swappiness. Recommandations Windows : mode d'alimentation, démarrage rapide |

### V1.0 « Stable »
- Installeurs **signés** (Windows : signature de code) et **mises à jour automatiques**.
- Extensions publiées sur **addons.mozilla.org**, le **Chrome Web Store** et **Edge Add-ons**.
- Site de documentation, guide de contribution des règles et des traductions.
- Audit de sécurité des actions de suppression.

### Idées pour après la V1.0 (hors périmètre)
- **macOS** (13+) : installeur signé et notarisé, CI `macos-latest`. Le code reste portable derrière les traits de `platform` ; rouvrir selon la demande et la disponibilité d'un testeur.
- Historique de santé du PC dans le temps (graphiques).
- Détection des doublons (intégration de Czkawka).
- Groupes d'onglets et sessions sauvegardées.
- Mode « une session de travail » : rouvrir seulement les onglets d'un projet.

---

## 7. Exigences fonctionnelles détaillées

### 7.1 `diagnose`
- **D1.** Le diagnostic complet prend **moins de 10 secondes**, sans compter l'analyse du disque, qui est lancée à part.
- **D2.** Il produit **au plus 5 recommandations**, classées par impact, chacune avec : un titre en langage simple, le gain estimé, le niveau de risque, un bouton d'action ou d'explication.
- **D3.** Les processus sont **regroupés par logiciel** (« VS Code : 3,2 Go », pas « code », « code », « code »…).
- **D4.** Seuils par défaut : disque > 90 % = alerte ; **espace libre < 2 Go = alerte critique** (bandeau, G7) ; swap > 50 % avec une RAM disponible < 15 % = alerte ; température > 80 °C au repos = alerte. Tous réglables.
- **D5.** **Chauffe par paliers.** Breathly croise température et charge. *Chaud sous forte charge* : normal, il le dit. *Chaud avec un processus gourmand* : cause logicielle, il le désigne et propose d'en parler. *Chaud sans charge* : cause matérielle probable, il affiche une liste de vérifications en langage simple (aération, poussière, mode d'alimentation, ventilateur). Si cela persiste (sur 3 relevés consécutifs du même diagnostic, valeur de départ non vérifiée), il suggère de **consulter un technicien**. Il ne promet aucune réparation, ne dit jamais qu'un composant est abîmé, et affiche « non disponible » si la température manque. Seuils de départ à valider sur du matériel varié (non vérifié).
- **D6.** **Processus et services système « à examiner ».** Un processus ou service qui consomme beaucoup sans être reconnu, ou lancé depuis un emplacement inhabituel, est listé avec une explication et un renvoi vers l'antivirus du système (Microsoft Defender, ClamAV). Breathly ne dit jamais « virus » ni « sain », et ne supprime rien de lui-même.
- **D7.** **Réglages du système.** Si le swap est saturé et qu'un swap compressé (zram) est absent, la recommandation l'explique et affiche la commande exacte à copier (mode guidé). L'application automatique est en V0.4.

### 7.2 `disk-scan` / `disk-clean`
- **S1.** Analyse du dossier personnel (~500 000 fichiers) en **moins de 60 s** sur un SSD.
- **S2.** Chaque élément proposé au nettoyage provient **d'une règle explicite** (fichier TOML). Aucune heuristique ne propose de supprimer un fichier. Les gros fichiers 🔴 sont affichés pour information avec « Ouvrir le dossier » : Breathly ne les supprime pas (US-022). Les dossiers de projets recréables sont reconnus par une règle « nom + fichier marqueur » (US-023).
- **S3.** Les fichiers de l'utilisateur vont dans la **corbeille** (crate `trash`). Les seules suppressions définitives sont les caches 🟢 et le **vidage confirmé de la corbeille** (US-037), qui liste et chiffre ce qu'il détruit ; l'écran l'indique.
- **S4.** **Liste d'exclusion** gérée par l'utilisateur, qui prime sur toutes les règles.
- **S5.** **Garde-fous non désactivables :** jamais en dehors des emplacements prévus par une règle ; jamais les profils de navigateur, `Documents`, `Images`, `Vidéos`, `Musique`, `Bureau`, les conteneurs chiffrés (`.hc`, `.vc`, `.kdbx`…), les dossiers `.git`, les fichiers ouverts par un programme.
- **S6.** **Historique** : date, élément, taille, destination (corbeille ou suppression définitive), et annulation pour ce qui est passé par la corbeille. Conservation 12 mois (valeur de départ) ; l'utilisateur n'efface que les entrées qui ne sont plus annulables (US-040).

### 7.3 `browser-guard`
- **B1.** Un navigateur lancé est détecté en **moins de 2 s**.
- **B2.** Les processus « en arrière-plan » (Edge *Startup Boost*, Chrome « continuer en arrière-plan ») ne déclenchent **pas** d'alerte tant qu'aucune fenêtre n'est ouverte, pour éviter les fausses alertes.
- **B3.** Fermer un navigateur se fait **proprement** (fermeture normale, pour que le navigateur puisse restaurer sa session), jamais en tuant directement le processus.
- **B4.** La liste des navigateurs (noms de processus par OS, nom affiché, icône) est un **fichier TOML** que les contributeurs peuvent compléter.

### 7.4 `tab-sleep`
- **T1.** Un onglet mis en veille garde sa position, son épinglage, son titre et son icône.
- **T2.** Ne jamais mettre en veille : l'onglet actif, un onglet `audible`, un onglet qui partage le micro, la caméra ou l'écran, un onglet protégé, un onglet avec un formulaire en cours de saisie quand le navigateur le signale.
- **T3.** Les « onglets du quotidien » sont calculés **sur la machine**, à partir de la fréquence d'activation sur 7 jours glissants. **Les URL ne sortent jamais du navigateur** : seuls le domaine et des compteurs sont transmis à l'application.
- **T4.** L'utilisateur peut voir, modifier ou effacer les données d'apprentissage.

### 7.5 `daemon`
- **M1.** Un seul processus en arrière-plan, **< 15 Mo de RAM** (objectif < 8 Mo), **< 0,5 % de CPU** en moyenne.
- **M2.** Le service Breathly peut être mis en pause depuis l'icône (« Mettre Breathly en pause 1 h »).

### 7.6 `app-watch`
- **A1.** Un logiciel est signalé quand sa RAM ou son CPU dépasse un seuil réglable **et** qu'il est inutilisé depuis la durée choisie (15 min par défaut).
- **A2.** La notification propose « Fermer », « Garder » et « Ne plus demander » ; la fermeture est propre (B3) et jamais automatique.
- **A3.** Liste blanche gérée par l'utilisateur ; les logiciels en cours d'enregistrement, de lecture ou d'appel ne sont jamais signalés.
- **A4.** Notification « moins de 2 Go libres » au plus une fois par jour, avec le bouton de nettoyage.

### 7.7 `system-tune`
- **Y1.** Chaque réglage est une règle explicite (TOML) : OS, distribution, paquet, effet attendu, retour arrière.
- **Y2.** Avant d'appliquer : liste exacte des changements et des paquets installés, confirmation, journal dans l'historique.
- **Y3.** Un réglage non vérifié pour la distribution détectée n'est pas proposé ; Breathly affiche seulement le conseil.

### 7.8 Droits administrateur (élévation)
- **E1.** Breathly ne tourne jamais en administrateur. Une action qui l'exige est exécutée seule, par une élévation ponctuelle de l'OS (UAC sous Windows, polkit sous Linux).
- **E2.** Avant la demande, un écran explique **pourquoi** ce droit est nécessaire, ce qui sera modifié et ce que cela libère.
- **E3.** **Mode guidé.** Si l'élévation est refusée ou indisponible, Breathly affiche la commande exacte à copier, avec la même explication, et n'exécute rien.
- **E4.** Breathly ne reçoit, ne stocke ni ne transmet jamais un mot de passe : la saisie reste dans la fenêtre de l'OS.
- **E5.** Les actions administrateur forment une liste fermée de règles (S2). Les garde-fous S5 s'appliquent aussi.

### 7.9 Machines à états
Les cycles de vie de l'analyse, du nettoyage et de l'entrée d'historique (V0.1) sont décrits, avec leurs conditions et leurs invariants, dans [`docs/design/0001-machines-a-etats.md`](design/0001-machines-a-etats.md). Les autres machines s'écrivent avant leur version.

---

## 8. Règles de nettoyage de départ (V0.1)

Exemple de règle (format complet, ses quatre formes et les chemins autorisés : [`design/0004-format-des-regles-de-nettoyage.md`](design/0004-format-des-regles-de-nettoyage.md)) :

```toml
[[rule]]
id          = "npm-cache"
name.fr     = "Cache de npm"
name.en     = "npm cache"
explain.fr  = "Paquets téléchargés par npm. Ils sont retéléchargés automatiquement si besoin."
explain.en  = "Packages downloaded by npm. They are re-downloaded automatically when needed."
risk        = "safe"          # safe | review | personal
os          = ["linux", "windows"]
kind        = "folder"        # folder | marked-folder | files-by-extension | system-trash
paths.linux   = ["~/.npm/_cacache"]
paths.windows = ["%LOCALAPPDATA%/npm-cache/_cacache"]
command     = "npm cache clean --force"   # optionnel, informatif en V0.1 : affiché en mode avancé, jamais exécuté (US-060.4)
```

**Règles minimales pour la V0.1 :**
- **Communes :** caches npm, pnpm, yarn, pip, Gradle, Cargo (registre), Go (build), Playwright, ms-playwright ; caches des navigateurs (**le cache seulement, jamais le profil**) ; caches de VS Code et de Cursor (VSIX en cache, `CachedData`) ; anciennes versions d'outils (Codex, etc.) ; installeurs dans Téléchargements (`.deb`, `.exe`, `.msi`, `.AppImage`, `.dmg`) en 🟡 ; dossiers de projets recréables en 🟡.
- **Linux :** `~/.cache` (par sous-dossier connu), corbeille (vidage confirmé, 🟡), miniatures.
- **Windows :** `%TEMP%`, corbeille (vidage confirmé, 🟡), `%LOCALAPPDATA%\CrashDumps`, miniatures, ancien dossier `Windows.old` (V0.4, droits administrateur).

---

## 9. Exigences non fonctionnelles

| Catégorie | Exigence |
|---|---|
| **Légèreté** | Budgets de l'ADR 0008 : service Breathly < 15 Mo de RAM ; **fenêtre ouverte ≤ 350 Mo de PSS** (mesuré à 288 Mo avec une page vide ; la cible initiale de 150 Mo était hors de portée avec le moteur web, valeur de départ re-mesurée à chaque version) ; installeurs `.deb` et `.rpm` < 15 Mo ; aucun processus de l'interface quand la fenêtre est fermée. La CI contrôle la taille des installeurs ; la mémoire de la fenêtre se mesure à la main sur les machines de validation. Conception : mesures échantillonnées, une seule boucle de surveillance, aucune activité quand la fenêtre est fermée en V0.1 |
| **Accessibilité** | WCAG 2.2 AA : contraste, navigation au clavier, lecteur d'écran, textes à taille réglable |
| **Simplicité** | Un utilisateur non technique libère de l'espace **en 3 clics ou moins** depuis l'accueil. Mode simple par défaut, mode avancé à la demande |
| **Langues** | Toutes les chaînes dans des fichiers de traduction (FR, EN au départ) ; aucun texte en dur dans le code |
| **Confidentialité** | Aucune connexion réseau, sauf la vérification des mises à jour (V1.0, désactivable) |
| **Sécurité** | Les actions administrateur passent par l'élévation standard de l'OS (UAC, polkit/pkexec), une action à la fois, jamais Breathly entier en administrateur ; mode guidé en repli (§7.8) |
| **Fiabilité** | Une erreur pendant un nettoyage n'interrompt pas le reste ; chaque erreur est affichée avec une explication |

---

## 10. Architecture et pile technique

```
┌──────────────────────── Navigateur ────────────────────────┐
│  Extension Breathly (WebExtension, TypeScript)             │
│  tabs.query · tabs.discard · tabs.onActivated · storage    │
└──────────────┬─────────────────────────────────────────────┘
               │ native messaging (stdin/stdout JSON)
┌──────────────▼──────────────┐      IPC locale       ┌───────────────────────┐
│ breathly-nmhost (Rust)      │ ◄──────────────────►  │ breathly-daemon (Rust)│
└─────────────────────────────┘ (socket Unix / pipe   │ icône · guard · stats │
                                 nommé Windows)        └──────────┬────────────┘
                                                                  │ IPC
                                                       ┌──────────▼────────────┐
                                                       │ Breathly (Tauri v2)   │
                                                       │ Rust + interface web  │
                                                       └───────────────────────┘
```

| Couche | Choix | Raison |
|---|---|---|
| Cœur | Rust (édition 2024), workspace Cargo | Faible consommation mémoire, binaire unique, multiplateforme |
| Système | `sysinfo` (processus, RAM, disques, températures), `jwalk` (parcours parallèle du disque), `trash` (corbeille multiplateforme) | Bibliothèques maintenues et multiplateformes |
| Interface | Tauri v2 + Svelte 5 (ADR 0001) | Design soigné, contributeurs web nombreux, utilise le moteur web du système (WebView2, WebKitGTK, WKWebView) |
| Extension | TypeScript, Manifest V3, construite pour Firefox et Chromium | Une seule base de code |
| Règles | Fichiers TOML dans `rules/` | Les contributeurs ajoutent des caches sans écrire de Rust |

---

## 11. Structure du dépôt (prévue ; figée par l'ADR 0010)

Seul ce que la version en cours utilise existe : en V0.1, le workspace, les crates `platform`, `rules`, `diagnose`, `disk` et `apps/desktop` ; les autres crates et `extension/` sont créés quand leur version démarre.

```
breathly/
├── crates/
│   ├── breathly-platform/    # module platform
│   ├── breathly-rules/       # module rules
│   ├── breathly-diagnose/    # module diagnose
│   ├── breathly-disk/        # modules disk-scan + disk-clean
│   ├── breathly-guard/       # module browser-guard
│   ├── breathly-watch/       # module app-watch (V0.2)
│   ├── breathly-startup/     # module startup (V0.4)
│   ├── breathly-tune/        # module system-tune (V0.4)
│   ├── breathly-elevated/    # assistant d'élévation à liste fermée (V0.4, ADR 0006)
│   ├── breathly-daemon/      # binaire du service (module daemon)
│   └── breathly-nmhost/      # binaire native messaging (module tab-sleep)
├── apps/desktop/             # interface Tauri (module app-ui)
│   ├── src-tauri/
│   └── ui/
├── extension/                # extension navigateur (module tab-sleep)
├── rules/                    # règles de nettoyage TOML
├── browsers/                 # définitions des navigateurs TOML
├── locales/                  # traductions (fr, en…)
├── docs/
│   ├── PRD.md                # ce document
│   └── specs/                # SPEC-<module-id>.md
├── tasks/                    # plan.md, todo.md
├── LICENSE · CONTRIBUTING.md · CODE_OF_CONDUCT.md · SECURITY.md
└── README.md
```

## 12. Commandes de développement (prévues)

```bash
cargo build --workspace                     # compiler tout le cœur
cargo test --workspace                      # tests unitaires et d'intégration
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo tauri dev        # (dans apps/desktop) lancer Breathly en développement
cargo tauri build      # installeurs
pnpm --dir extension build                  # construire les extensions Firefox et Chromium
pnpm --dir extension test
npx web-ext lint --source-dir extension/dist/firefox
```

## 13. Stratégie de test

- **Règles :** chaque règle TOML est validée automatiquement (schéma, chemins, explication FR et EN présentes, niveau de risque).
- **Nettoyage :** tests d'intégration **uniquement dans des dossiers temporaires** (`tempfile`). **Aucun test ne touche le vrai dossier personnel.** Les garde-fous S5 ont chacun un test dédié qui vérifie le refus.
- **Plateformes :** CI GitHub Actions sur `ubuntu-latest` et `windows-latest` (`macos-latest` s'ajoutera avec le support de macOS, après la V1.0).
- **Extension :** tests unitaires des règles de mise en veille (Vitest) ; tests manuels sur Firefox, Zen, Brave et Chrome à chaque version.
- **Validation réelle :** avant chaque version, test sur les machines de validation (Linux du développeur, Windows de l'ami, Ubuntu de l'amie) et essai de l'interface par une personne de chaque profil d'usage (§2).

## 14. Règles de travail

- **Toujours :** simulation avant suppression dans le code comme dans l'interface ; tests avant chaque commit ; `clippy` sans avertissement ; textes dans les fichiers de traduction.
- **Demander d'abord :** ajout d'une dépendance ; nouvelle règle 🟢 qui supprime définitivement ; tout ce qui demande des droits administrateur ; changement du format des règles.
- **Jamais :** supprimer en dehors d'une règle ; écrire un test qui touche le vrai dossier personnel ; envoyer des données hors de la machine ; tuer un navigateur sans lui laisser le temps de se fermer proprement.

---

## 15. Critères de réussite

| Version | Critère vérifiable |
|---|---|
| **V0.1** | Sur la machine Windows de l'ami, une personne non technique libère ≥ 5 Go (espace réellement libéré, corbeille vidée comprise) **en ≤ 3 clics, sans aide** ; aucun fichier hors règle n'est touché (vérifié par l'historique) ; installeur < 15 Mo |
| **V0.1** | Sur la machine de référence Linux, Breathly retrouve **au moins 80 %** des ~30 Go libérés à la main le 2026-10-17 |
| **V0.1** | Quand l'espace libre est < 2 Go, le bandeau d'alerte apparaît à l'ouverture et mène au nettoyage en un clic ; sur une machine chaude sans charge, le diagnostic dit « cause matérielle probable » sans promettre de réparation |
| **V0.2** | L'ouverture d'un 2e navigateur déclenche la fenêtre d'alerte en < 2 s ; service Breathly < 15 Mo de RAM sur 24 h (cible, voir §9) ; un logiciel gourmand inutilisé depuis 15 min est signalé, jamais fermé sans clic |
| **V0.3** | Avec ~40 onglets dont des épinglés, la RAM du navigateur baisse d'**au moins 40 %** après 30 min, **sans qu'aucun onglet ne disparaisse** |
| **V0.4** | Les services de développement lancés au démarrage sont détectés et désactivables depuis l'interface, avec possibilité de les réactiver ; sur une distribution vérifiée, zram est installé avec élévation ponctuelle et un retour arrière fonctionne |
| **V1.0** | Installeurs signés pour Windows et Linux ; extensions publiées sur les 3 boutiques ; ≥ 3 contributeurs externes ont ajouté des règles ou des traductions |

---

## 16. Risques

| Risque | Impact | Parade |
|---|---|---|
| **Navigateurs en flatpak ou en snap** (Zen en flatpak, Firefox en snap sur Ubuntu) : le *native messaging* est limité par le bac à sable | L'extension ne peut pas parler à Breathly | Prototype dès le début de la V0.3 ; portail `xdg-desktop-portal` (WebExtensions) quand il est disponible ; sinon mode autonome de l'extension (T, §6 V0.3) |
| Zen / forks de Firefox : emplacement non standard des manifestes *native messaging* | Installation incomplète | Définitions par navigateur dans `browsers/*.toml`, testées dans la CI |
| Une règle de nettoyage fausse supprime des données | Perte de confiance, irréversible | Règles revues par 2 personnes ; garde-fous S5 non désactivables ; corbeille pour les fichiers de l'utilisateur |
| Antivirus Windows qui bloque un exécutable non signé | L'ami ne peut pas installer | Signature de code dès que possible ; documentation SmartScreen en attendant |
| WebKitGTK lent ou ancien sur certaines distributions | Interface lente sous Linux | Interface sobre, pas d'animations lourdes ; test sur Ubuntu LTS |
| **Détection d'« inutilisé »** non uniforme (Wayland, Windows) | Fausses alertes ou logiciel utile signalé | Prototype avant V0.2 ; ne jamais fermer sans clic ; liste blanche ; définition prudente (fenêtre, son, CPU) |
| **Faux sentiment de sécurité** : « à examiner » compris comme « sain » ou « virus » | L'utilisateur croit son PC protégé ou panique | Vocabulaire fixé (D6) ; renvoi vers l'antivirus ; jamais « sain » ni « virus » |
| **Réglages système différents selon la distribution** (zram, paquets) | Commande fausse ou paquet absent | Vérification par distribution avant la règle (Y3) ; pas de réglage non vérifié |
| **Surchauffe mal diagnostiquée** | Conseil inutile ou alarmiste | Paliers prudents (D5) ; « non disponible » plutôt qu'une valeur fausse ; jamais de promesse de réparation |
| Températures indisponibles (sous Windows, souvent sans droits administrateur) | Diagnostic incomplet | Afficher « non disponible » plutôt qu'une valeur fausse |

---

## 17. Questions ouvertes

1. ~~**Framework de l'interface**~~ → **Svelte 5** ✅ (décision du 2026-10-17, ADR 0001 ; à confirmer par le prototype d'un écran)
2. ~~**Licence**~~ → MIT ✅
3. ~~**macOS**~~ → **après la V1.0** ✅
4. ~~**Hébergement**~~ → https://github.com/AresGn/Breathly ✅
5. ~~**Identité visuelle**~~ → **maquette cliquable de 4 écrans** (accueil avec bandeau d'espace faible, diagnostic, disque, onglets ; mode simple et avancé) dans `docs/design/`, validée avant d'écrire l'interface ✅ (livrées le 2026-10-18 : `docs/design/maquettes/`)
6. **Seuils de chauffe** (D5) : valeurs de départ à valider sur du matériel varié (portable, fixe, Windows sans droits).
7. **Définition de « inutilisé »** (A1) : à prototyper sur X11, Wayland et Windows (étape 2.9).
8. **Paquets et distributions** pour `system-tune` (Y3) : zram-tools, zram-generator, etc. à vérifier dans la documentation de chaque distribution avant la V0.4.
9. **Budgets de légèreté** (G5, §9) : à mesurer par le prototype puis à fixer par ADR.
