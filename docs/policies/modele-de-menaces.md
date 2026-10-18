# Modèle de menaces de Breathly

- **Statut** : à relire (étape 2.8 du parcours)
- **Date** : 2026-10-18
- **Méthode** : STRIDE sur les 5 flux les plus critiques, une fiche chacun.
- **Question posée à chaque flux** : *« Que se passe-t-il si l'interface web est compromise, si un logiciel malveillant tourne déjà avec le compte de l'utilisateur, ou si un composant tiers est piégé ? »*
- **Posture décidée le 2026-10-18** : Breathly **ne sert pas d'amplificateur**. Empêcher un logiciel malveillant de lire ou modifier les fichiers de l'utilisateur est hors périmètre (il le peut déjà) ; l'empêcher de se servir de Breathly pour faire plus est dans le périmètre.
- Les règles `SEC1` à `SEC11` sont dans [`securite.md`](securite.md) ; les états dans [`../design/0001-machines-a-etats.md`](../design/0001-machines-a-etats.md).

## Frontières de confiance

```
 Utilisateur ── clic ──► Interface (webview, NON fiable) ──commandes Tauri──► Cœur Rust (de confiance)
                                                                                 │
                              fichiers locaux (réglages, exclusion, historique : NON fiables au chargement)
                                                                                 │
 Navigateur ◄─native messaging─► Hôte ◄─socket/tube─► Service Breathly ──(V0.2+)   Assistant élevé (V0.4)
 (page web : NON fiable)                                                           (liste fermée)
```

## Flux critiques

1. Suppression de fichiers (V0.1)
2. Règles et fichiers locaux (V0.1)
3. Élévation des droits administrateur (V0.4)
4. Extension, hôte et service (V0.2-V0.3)
5. Distribution et dépendances (toutes versions)

---

### 1. Suppression de fichiers

- **Acteurs et frontières** : l'utilisateur confirme dans l'interface ; l'interface appelle le cœur ; le cœur supprime ou envoie à la corbeille. Seul le cœur est de confiance.
- **Actifs** : les fichiers de l'utilisateur (documents, projets, profils de navigateur).

| STRIDE | Menace | Parade | Test qui la prouve |
|---|---|---|---|
| Spoofing | Un script de l'interface simule la confirmation de l'utilisateur | jeton de confirmation lié à une simulation, à usage unique (SEC3) | rejouer un jeton ; jeton d'une autre sélection |
| Tampering | Le chemin change entre la vérification et l'action (lien symbolique, jonction) | normalisation, liens non suivis, garde-fous revérifiés par élément (SEC2) | test de lien symbolique et de jonction vers `Documents` |
| Repudiation | « Breathly a supprimé mon fichier » sans preuve | entrée « pending » avant l'action (SEC4) | historique non inscriptible : aucune suppression |
| Information disclosure | Chemins personnels dans les journaux | journaux par liste blanche (SEC9) | scan de la sortie de journal |
| Denial of service | Nettoyage énorme qui gèle le PC ; historique qui grossit | arrêt après l'élément en cours, fils bornés, rotation 12 mois | test d'arrêt, test de rotation |
| Elevation of privilege | L'interface demande de supprimer un chemin arbitraire (`Documents`) | l'interface n'envoie que des identifiants (SEC3) ; garde-fous non désactivables (SEC2, SEC5) | commande appelée avec un chemin : refusée ; un test par garde-fou |

### 2. Règles et fichiers locaux

- **Acteurs et frontières** : les règles viennent des contributeurs (PR) et sont intégrées au binaire ; réglages, exclusion et historique sont lus depuis le dossier de l'utilisateur.
- **Actifs** : la cohérence des garde-fous, l'étendue de ce que Breathly peut supprimer.

| STRIDE | Menace | Parade | Test |
|---|---|---|---|
| Spoofing | Une règle piégée proposée dans une PR comme « cache » | relecture par deux personnes, validation CI des chemins (SEC6) | règle avec chemin hors dossier autorisé : refusée |
| Tampering | Un logiciel malveillant édite le fichier de réglages ou d'exclusion pour élargir un nettoyage | validation au chargement, exclusion qui ne fait que restreindre, garde-fous non désactivables (SEC5) | fichiers forgés : refus avec message |
| Repudiation | Historique altéré pour cacher une action | **hors périmètre**, documenté : l'historique n'est pas infalsifiable (posture) | — |
| Information disclosure | Réglages ou historique lus par un autre compte | droits limités à l'utilisateur (SEC5) | vérification des droits à la création |
| Denial of service | Fichier de réglages énorme ou invalide qui empêche de démarrer | taille maximale, repli sur les valeurs par défaut avec message (SEC5) | fichier de 100 Mo, fichier tronqué |
| Elevation of privilege | Champ `command` d'une règle qui lance un programme | `command` jamais exécuté (SEC6, US-060.4) | règle avec `command` : rien lancé |

### 3. Élévation des droits administrateur (V0.4)

- **Acteurs et frontières** : Breathly (utilisateur) lance l'assistant `breathly-elevated` par polkit (Linux) ou UAC (Windows). L'assistant est le seul composant privilégié.
- **Actifs** : les droits administrateur de la machine.

| STRIDE | Menace | Parade | Test |
|---|---|---|---|
| Spoofing | Un faux assistant remplace le vrai binaire | action polkit liée au chemin installé ; binaire installé hors dossier modifiable par l'utilisateur ; signature à la V1.0 (SEC7, SEC11) | installation avec droits de fichier contrôlés |
| Tampering | Arguments ou environnement forgés pour faire exécuter autre chose | identifiant de règle de la liste fermée, paramètres revalidés ; pkexec ne valide pas les arguments (SEC7) | arguments forgés refusés |
| Repudiation | Action administrateur sans trace | inscrite à l'historique par l'interface avant lancement (SEC4) | test d'ordre : pending avant élévation |
| Information disclosure | Mot de passe intercepté | Breathly ne manipule jamais de mot de passe : saisie dans la fenêtre de l'OS (SEC7) | revue du code : aucune saisie de mot de passe |
| Denial of service | Boucle de demandes d'élévation qui harcèle l'utilisateur | une action à la fois, après une explication (PRD E1, E2) | test : pas de seconde demande sans clic |
| Elevation of privilege | Un logiciel malveillant se sert de l'assistant pour agir en administrateur | liste fermée de règles, aucune commande libre, authentification administrateur à chaque action (SEC7) | identifiant inconnu : refusé |

### 4. Extension, hôte et service (V0.2-V0.3)

- **Acteurs et frontières** : pages web (non fiables) → extension → hôte de messagerie native → service Breathly.
- **Actifs** : la liste des domaines d'onglets, la capacité de mettre des onglets en veille ou de fermer un navigateur.

| STRIDE | Menace | Parade | Test |
|---|---|---|---|
| Spoofing | Une autre extension se fait passer pour Breathly auprès de l'hôte | extensions autorisées nommées exactement, sans joker (SEC8) | origine non déclarée : refusée |
| Tampering | Messages forgés vers le service (autre compte, autre processus) | socket aux droits de l'utilisateur, messages versionnés et validés (SEC8) | connexion d'un autre compte : refusée ; message mal formé |
| Repudiation | Fermeture d'un navigateur contestée | action inscrite ; fermeture seulement sur clic (invariant 10) | test d'historique |
| Information disclosure | Fuite d'URL d'onglets | seuls le domaine et des compteurs transitent (SEC9, PRD T3) | test sur le contenu des messages |
| Denial of service | Message énorme ou rafale de messages | taille bornée, débit limité côté hôte (SEC8) | message de 2 Mo : refusé |
| Elevation of privilege | Une page web appelle l'hôte | le contenu d'une page n'accède pas au native messaging ; l'extension valide l'origine de l'expéditeur (documentation Chrome) | message d'un script de page : ignoré |

### 5. Distribution et dépendances

- **Acteurs et frontières** : mainteneurs → CI GitHub → Releases → utilisateur ; dépendances Cargo et npm.
- **Actifs** : la confiance dans l'installeur et dans le code livré.

| STRIDE | Menace | Parade | Test |
|---|---|---|---|
| Spoofing | Faux site ou fork qui distribue un faux Breathly | README pointe vers les Releases officielles ; empreintes SHA-256 publiées (SEC11) | procédure de vérification documentée et essayée |
| Tampering | Installeur ou dépendance trafiqués | empreintes ; build par la CI depuis un tag ; `cargo audit`, délai de 7 jours, ajout de dépendance demandé (SEC10) | étape de release de la CI ; audit en CI |
| Repudiation | Origine d'un build contestée | journal de la CI ; attestation de provenance à étudier (non vérifiée) | — |
| Information disclosure | Secret de signature ou jeton dans le dépôt | scan de secrets en CI (`make check`), secrets de signature dans la CI uniquement (SEC10) | `tools/scan-secrets.sh` sonde négative |
| Denial of service | Dépendance qui casse le build, action CI supprimée | versions verrouillées (`--locked`), actions épinglées par SHA (SEC10) | `pin-actions.py --check` |
| Elevation of privilege | Une dépendance introduit du code réseau ou un script d'installation | interdiction des bibliothèques réseau (SEC1) ; scripts d'installation `.deb` et `.msi` relus | contrôle de l'arbre de dépendances |

## Sortie attendue : tests de sécurité à écrire

Une ligne par parade ; chacune devient un critère de ticket (étape 6).

| # | Test | Parade | Version |
|---|---|---|---|
| TS-01 | Un test par garde-fou de S5 (Documents, Images, Vidéos, Musique, Bureau, profils de navigateur, conteneurs chiffrés, `.git`, fichiers ouverts) | SEC2 | V0.1 |
| TS-02 | Lien symbolique et jonction vers un emplacement interdit : non suivis | SEC2 | V0.1 |
| TS-03 | Une commande appelée avec un chemin est refusée ; seules les identifiants d'éléments passent | SEC3 | V0.1 |
| TS-04 | Jeton de confirmation : rejoué, expiré ou d'une autre sélection refusé | SEC3 | V0.1 |
| TS-05 | Inventaire des commandes Tauri : toute commande est listée et restreinte à la fenêtre principale | SEC3 | V0.1 |
| TS-06 | Historique non inscriptible : aucune suppression | SEC4 | V0.1 |
| TS-07 | Réglages, exclusion et historique forgés ou énormes : refus avec message, valeurs par défaut | SEC5 | V0.1 |
| TS-08 | La liste d'exclusion ne peut pas élargir un nettoyage ; aucun fichier ne désactive un garde-fou | SEC5 | V0.1 |
| TS-09 | Règle invalide, chemin hors dossier autorisé, `command` présent : refus ou ignoré, rien d'exécuté | SEC6 | V0.1 |
| TS-10 | Absence de bibliothèque réseau dans l'arbre de dépendances ; aucune connexion à l'exécution | SEC1 | V0.1 |
| TS-11 | Les journaux ne contiennent ni chemin complet ni contenu de fichier | SEC9 | V0.1 |
| TS-12 | `cargo audit` en CI ; scan de secrets ; actions épinglées | SEC10 | V0.1 |
| TS-13 | Les empreintes SHA-256 des installeurs sont publiées et vérifiables | SEC11 | première release |
| TS-14 | Assistant d'élévation : identifiant inconnu, arguments forgés, environnement hostile refusés | SEC7 | V0.4 |
| TS-15 | Socket : connexion d'un autre compte refusée ; messages mal formés ou de plus de 1 Mo refusés | SEC8 | V0.2 |
| TS-16 | Hôte de messagerie : origine non déclarée refusée ; message d'une page web ignoré | SEC8 | V0.3 |

## Questions ouvertes

1. Écart entre vérification et action lors de la suppression : stratégie exacte (ADR de sécurité, `securite.md` §3).
2. Politique de contenu (CSP) et capacités de Tauri : à lire dans la documentation avant d'écrire la configuration.
3. Attestation de provenance des builds : disponibilité et fonctionnement sur ce dépôt non vérifiés.
