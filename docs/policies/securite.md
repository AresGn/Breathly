# Sécurité par conception

- **Statut** : règles en vigueur ; à adapter par ADR (jamais à affaiblir en silence)
- **Portée** : toutes les applications du projet. Les invariants métier (cloisonnement, audit, données d'identité) sont dans `AGENTS.md` ; ce document ajoute les règles techniques qui les protègent.
- **Lecteurs** : toute personne ou tout agent qui écrit du code.

**Principe** : une règle n'existe que si elle a un garant, une vérification automatique ou une ligne de relecture. Chaque ligne ci-dessous dit lequel. Les écritures de code détaillées sont dans le `CODING.md` de chaque application. Toute l'API est traitée en **Zero Trust envers les clients** : tout ce qui vient d'un navigateur ou d'une application mobile est potentiellement forgé.

## 1. Règles en vigueur

| # | Règle | Garant |
|---|---|---|
| S1 | Toute route est derrière l'authentification, sauf la route de santé et les routes publiques déclarées. Une route ajoutée est une décision visible dans la PR. | test sur l'inventaire des routes |
| S2 | Chaque route sur une ressource cloisonnée a un test d'intrusion croisée : un acteur hors périmètre reçoit **404** (pas 403). Les ressources enfants sont liées à leur parent. | test par route, relecture |
| S3 | Les attributs assignables d'un modèle sont déclarés explicitement ; la rigueur de l'ORM (attributs écartés en silence, chargements paresseux) est active hors production. | test, relecture |
| S4 | **Journaux par liste blanche** : on ne journalise que des identifiants et des codes, jamais un objet entier, un texte libre, une donnée d'identité, un jeton. Format **JSON structuré** avec un identifiant de requête. | test qui joue des parcours et cherche les données interdites dans la sortie ; relecture |
| S5 | **Limitation de débit** nommée et testée sur chaque route sensible (connexion, inscription, code à usage unique, envoi). Valeurs en configuration (§2). | test par route (429) |
| S6 | Après des échecs répétés de connexion, **blocage temporaire croissant** du compte, en plus de la limitation de débit. | test |
| S7 | **Fichiers** : stockage privé, nom aléatoire, type vérifié sur le contenu et sur l'extension, taille bornée, jamais de dossier public. Téléchargement par URL signée courte (5 minutes ; 1 minute pour une pièce d'identité, dont la consultation est auditée avant d'être servie). | test, relecture |
| S8 | Un jeton de notification d'appareil est lié à la session et révoqué à la déconnexion. Le contenu d'une notification est neutre. | test |
| S9 | Débogage désactivé hors développement ; proxys et hôtes de confiance déclarés explicitement ; l'adresse IP d'une requête est une donnée non fiable, jamais seule base d'une décision de sécurité. | relecture du déploiement |
| S10 | Un tableau validé liste ses clés autorisées ; aucune entrée utilisateur dans une règle d'unicité qui l'ignore. | relecture |
| S11 | Mobile : aucun secret dans le binaire ; build de production obfusqué avec symboles séparés ; jeton dans le stockage sécurisé du système, données hors ligne sensibles dans une base chiffrée ; jamais de stockage non chiffré pour un jeton ou une donnée personnelle. | relecture |
| S12 | Dépendances : audit dans `make ci-local` ; mises à jour hebdomadaires par Dependabot avec délai ; une version publiée depuis moins de `MIN_RELEASE_AGE_DAYS` jours ne s'installe pas ; actions CI épinglées par SHA. | CI locale, Dependabot, `tools/pin-actions.py` |

## 2. Valeurs de départ (paramètres de configuration)

À valider avec le client avant la mise en service ; elles se changent sans modifier le code.

| Route | Limite proposée |
|---|---|
| Connexion | 5 par minute, par adresse IP et par identifiant |
| Code à usage unique | 3 par 10 minutes, par identifiant |
| Création sensible (envoi de signalement, paiement…) | 10 par heure, par compte |
| Blocage après échecs de connexion | N échecs (proposition : 5), puis 1, 5, 15 minutes |

## 3. Décisions à prendre par un ADR avant le premier sprint d'authentification

- **Modèle de jetons** (jeton unique à expiration, ou accès court plus rafraîchissement avec rotation), durée de vie, révocation par appareil.
- **Session d'une application web** (cookie `HttpOnly` ou jeton) : un jeton n'est jamais lisible par le JavaScript du navigateur.
- **Épinglage du certificat mobile** : après le choix de l'hébergement (un certificat renouvelé peut bloquer des utilisateurs).
- **Détection d'exécution compromise côté mobile** : dépendance supplémentaire, donc ADR.
- **Modélisation des menaces** (`policies/modele-de-menaces.md`) : à écrire avant ce sprint.

## 4. Pistes écartées (à remplir au fil des ADR)

| Piste | Raison |
|---|---|
| <ex. identifiants séquentiels exposés> | <non énumérables : UUID v7 / ULID> |
| <ex. paquet de permissions par rôle> | <invariant : permissions, pas rôles> |
| <ex. chiffrement par défaut du framework> | <sans donnée associée : service AEAD dédié> |

## 5. Sources vérifiées

Pour chaque affirmation sur un framework : la page de documentation lue, la version, la date. Ce qui n'est pas dans les guides mais dans le code source ou l'API de référence est signalé comme tel. Les points vérifiés **par essai** (projet jetable) sont datés.
