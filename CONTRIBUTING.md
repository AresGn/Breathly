# Contribuer à Breathly

Règles du dépôt, pour les humains et pour les agents. Courtes et vérifiables : chaque règle dit **qui** la fait respecter (la machine ou une personne). Les invariants métier et le style de code sont dans `AGENTS.md`, l'écriture du code de chaque application dans son `CODING.md` (§6) ; le démarrage du poste est dans `docs/guides/demarrage-developpeur.md`.

**Un projet qui manipule des données de personnes ne tolère pas qu'une règle soit contournée « pour gagner du temps » : c'est un défaut, pas un raccourci.**

## 1. Les règles non négociables

1. **Rien n'entre dans `main` sans PR, sans contrôle vert (`local-ci`, ou `gate` quand la CI hébergée est disponible) et sans l'approbation d'une autre personne que l'auteur.** Personne ne pousse directement sur `main`, hors documentation et décision du propriétaire du dépôt (variable `BREATHLY_ALLOW_MAIN`, voir §5). **Cette règle n'est pas imposée par GitHub** tant que le dépôt est privé sur un compte gratuit (§5) : elle repose sur le hook de pré-push et sur chacun.
2. **Le hook ne se contourne pas.** `git commit --no-verify` et `git push --no-verify` sont interdits. Si un contrôle local ne peut pas tourner sur votre poste (outil manquant), installez l'outil ; si le contrôle est faux, corrigez le contrôle dans une PR dédiée.
3. **Aucun secret dans le dépôt**, jamais, même « temporaire », même dans un test, même retiré au commit suivant (il reste dans l'historique).
4. **Aucune donnée réelle** : pas de vrai identifiant national, de vrai nom, de vrai numéro de téléphone, de vraie pièce d'identité, dans le code, les tests, les captures d'écran, les journaux ou les descriptions de PR.
5. **Une PR, un sujet.** Un correctif sans rapport va dans sa propre PR.
6. **On ne baisse pas la barre pour passer au vert** : pas de test sauté, supprimé ou affaibli ; pas de suppression de contrôle (`@ts-ignore`, `eslint-disable`, `// ignore`, `# noqa`, `#[allow]`) sans raison écrite sur la ligne ; pas de seuil de contrôle abaissé.
7. **Les invariants de `AGENTS.md` sont des défauts bloquants** : un import illégal entre couches ou modules, une requête sur une ressource cloisonnée sans filtre de périmètre, un test sur un nom de rôle, une vue publique obtenue en filtrant une vue interne, une donnée d'identité en clair dans une réponse ou un journal.

## 2. Flux de travail

```
branche  →  commits  →  push  →  PR (brouillon)  →  CI verte  →  relecture  →  fusion en « squash »
```

0. **Une PR = un ticket.** La vue globale de tous les tickets (qui a quoi, ce qui est bloqué, par quoi commencer) est dans [`docs/backlog.md`](docs/backlog.md), régénérée par `make backlog` ; GitHub reste la source de vérité. Chaque PR traite un ticket GitHub (`docs/agents/issue-tracker.md`), pas plusieurs, pas une fraction arbitraire. Prendre un ticket : se l'assigner (`gh issue edit <n> --add-assignee @me`) avant de commencer, et n'en prendre un que si ses bloqueurs sont fermés. Le corps de la PR contient `Closes #<n>` ; un ticket livré en plusieurs PR les relie par `Part of #<n>`, la dernière portant `Closes #<n>`. Un travail sans ticket (correctif ponctuel, documentation) reste une PR courte ; s'il grossit, on ouvre un ticket. Une PR qui déborde de son ticket est découpée.
1. **Partir de `main` à jour** : `git fetch origin main && git switch -c <branche> origin/main`.
2. **Une branche par ticket**, nommée `type/<numéro>-sujet-court` : `feat/32-schema-comptes`, `fix/41-curseur`, `docs/adr-cartographie` (sans ticket). Pas de branche qui vit plus de quelques jours : on découpe.
3. **Commiter souvent, petit.** Le hook de pré-commit tourne à chaque commit (§3).
4. **Pousser** : le hook de pré-push rejoue les contrôles sur tout ce qui diffère de `origin/main`.
5. **Ouvrir la PR en brouillon**, remplir `.github/pull_request_template.md` (checklists comprises), la passer « prête » quand la CI est verte.
6. **Répondre à chaque commentaire** de relecture (corriger, ou expliquer pourquoi non). Le relecteur résout ses fils, pas l'auteur.
7. **Fusion en « squash »** par l'auteur ou un relecteur, une fois la PR approuvée et le contrôle vert. Le titre de la PR devient le message du commit : il suit le format du §4.

On ne réécrit jamais l'historique d'une branche **partagée** (ni rebase, ni `--amend`, ni force-push) : pour la mettre à jour, fusionner `main` dedans. Sur **votre** branche personnelle, un `--force-with-lease` est permis (par exemple pour aplatir une branche contenant un secret retiré).

## 3. Contrôles : qui bloque quoi

| Où | Quand | Contrôle | Se contourne ? |
|---|---|---|---|
| Hook `pre-commit` | chaque commit | secrets, puis les portes des applications modifiées | oui (`--no-verify`) : interdit par la règle 2 |
| Hook `pre-push` | chaque push | secrets dans tout l'historique poussé, puis mêmes contrôles ; refuse le push direct sur `main` | oui : interdit |
| `make ci-local` | avant de passer la PR « prête » | tout ce que rejoue `gate`, puis publication du statut `local-ci` sur le commit | **non**, une fois la protection de branche activée (§5) ; sinon, c'est une règle de confiance |
| CI, job `gate` | chaque PR | tout (secrets, documentation, applications, audit) | **non**, une fois la protection de branche activée et les minutes de la CI disponibles (§5) |
| Scanner de secrets tiers (GitGuardian, Gitleaks) | chaque PR | secrets | non |
| Relecture humaine | chaque PR | conception, sécurité, lisibilité, test du bon comportement | non : approbation requise |

- Activer les hooks une fois par clone : `make hooks` (compris dans `make setup`).
- `make check` rejoue localement les portes de chaque application ; les contrôles propres au projet (base de données, contrat d'API) sont dans `tools/extra-checks.sh`.
- Un commit de documentation seule ne déclenche que le contrôle des secrets.
- **CI locale (`make ci-local`)** : tant que la CI hébergée est indisponible, c'est la porte de fusion. Le script exige un arbre propre et un commit déjà poussé, rejoue les secrets (fichiers et historique), l'audit des dépendances, toutes les applications, la documentation, puis publie le statut `local-ci` sur le commit. Il ne publie rien si un contrôle échoue ou s'il a modifié un fichier. Chaque nouveau commit demande de relancer `make ci-local` : le statut est lié à un commit précis. `bash tools/ci-local.sh --no-publish` fait un essai sans publier.
- Limite connue : toute personne avec accès en écriture peut publier ce statut sans avoir lancé les contrôles. La confiance et la relecture humaine compensent ; publier `local-ci` sans l'avoir obtenu vert en local est une faute grave.

## 4. Commits et PR

Format du message (français, impératif ou participe, ≤ 72 caractères pour la première ligne) :

```
type(portée): objet
```

- **Types** : `feat`, `fix`, `docs`, `refactor`, `test`, `ci`, `chore`, `perf`.
- **Portées** : le nom d'une application, d'un module, ou `db`, `infra`, `contract`, `design`, `prd`, `adr`.
- Le corps (facultatif) explique **pourquoi**, pas ce que le diff montre déjà.
- Un commit = un changement qui passe les contrôles seul.

Une PR :
- dit **quoi et pourquoi** en 2 ou 3 lignes, avec le lien de l'ADR ou de la section du PRD concernée ;
- reste lisible : au-delà d'environ 400 lignes de code utile, la découper (les fichiers générés ou les verrous de dépendances ne comptent pas) ;
- dit **ce qui n'a pas été vérifié** si c'est le cas ;
- ne mélange jamais refactoring et changement de comportement.

## 5. Protection de `main` (à régler une fois, par le propriétaire du dépôt)

**État au 2026-10-18 : appliquée** (dépôt public, réglée par le propriétaire via l'API GitHub). Sur `main` :
- PR obligatoire, **0 approbation requise** tant que le projet n'a qu'un mainteneur (une approbation requise bloquerait la seule personne qui peut fusionner) ; à passer à **1** dès qu'un second mainteneur existe ;
- contrôle `gate` obligatoire et branche à jour avant fusion ;
- historique linéaire, conversations résolues, aucun force push, aucune suppression ;
- la règle vaut aussi pour les administrateurs (le propriétaire passe par une PR comme les autres) ;
- fusion **uniquement par squash**, branche supprimée après fusion.

Le hook de pré-push local reste utile : il refuse un push sur `main` avant même d'atteindre GitHub.

Les hooks sont locaux et contournables. Seul GitHub peut rendre les règles 1 et 2 impossibles à enfreindre. Réglage à faire dans **Settings** (les libellés de GitHub évoluent ; la règle et son effet comptent plus que le chemin exact).

**Branches** (Settings > Branches > Add branch protection rule, ou Settings > Rules > Rulesets) pour la branche `main` :

- [ ] **Require a pull request before merging**
  - [ ] Require approvals : **1**
  - [ ] Dismiss stale pull request approvals when new commits are pushed
  - [ ] Require approval of the most recent reviewable push (l'auteur du dernier push ne s'approuve pas)
- [ ] **Require status checks to pass before merging** : cocher **`local-ci`** (et **`gate`** dès que la CI hébergée tourne). Un statut publié par `gh api` est requis comme un check : il apparaît dans la liste après sa première publication
  - [ ] Require branches to be up to date before merging
- [ ] **Require conversation resolution before merging**
- [ ] **Do not allow bypassing the above settings** (inclut les administrateurs, donc le propriétaire)
- [ ] **Block force pushes** et **Restrict deletions**

**Fusion** (Settings > General > Pull Requests) :
- [ ] Autoriser **uniquement « Squash merging »**
- [ ] **Automatically delete head branches**

**À ne pas oublier**
- Un contrôle ne peut être choisi comme « requis » que s'il s'est exécuté récemment sur le dépôt. **N'activer l'exigence de `gate` qu'après un premier run vert**, sinon plus personne ne peut fusionner.
- Tant que les minutes de la CI hébergée sont épuisées, **`gate` ne doit pas être exigé** (il ne se terminerait jamais) : exiger `local-ci` à la place, après l'avoir publié une première fois sur une PR.
- Le job `gate` accepte les jobs sautés (application pas encore initialisée) et refuse un job en échec ou annulé.
- Vérifier que le réglage tient : essayer de pousser sur `main` depuis un clone doit être refusé.

## 6. Règles de code (résumé de `AGENTS.md`)

Le détail d'écriture de chaque application est dans son `CODING.md` (chemins dans `.kit.conf`). Chacun finit par une « définition de terminé » que l'auteur vérifie avant d'ouvrir la PR.

| Sujet | Règle | Contrôlé par |
|---|---|---|
| Architecture | couches et modules : le domaine n'importe ni framework ni SDK tiers ; un module ne touche pas aux classes internes d'un autre | outil d'architecture de la pile, tests d'architecture |
| Permissions | `canUser($user, 'action:ressource')`, jamais un nom de rôle | relecture, test |
| Cloisonnement | toute requête sur une ressource cloisonnée filtrée par périmètre | relecture, tests de l'API |
| Typage | pas de type flou hors frontière ; entrées validées | analyse statique de la pile |
| Taille | fonction ≤ 40 lignes, imbrication ≤ 3, ≤ 3 paramètres positionnels, fichier ≤ 250 à 300 lignes | machine puis relecture |
| Erreurs | jamais de capture vide ; échec métier attendu = valeur de retour typée | relecture |
| États | énumération et traitement exhaustif, sans cas par défaut | analyse statique |
| Journaux | aucune donnée personnelle | relecture, tests |
| Contrat d'API | jamais écrit à la main ; régénéré puis commité | CI |
| Base de données | une contrainte critique est une contrainte, pas seulement du code ; conventions de `AGENTS.md` | relecture, tests de schéma |

## 7. Tests

- Tout comportement nouveau ou corrigé a un test qui échoue sans lui. Un bug se corrige en écrivant d'abord le test qui le reproduit.
- On teste le **comportement observable** (l'API en boîte noire contre une vraie base, la machine à états du domaine), pas l'implémentation.
- Un test ne dépend ni de l'ordre d'exécution, ni de l'heure réelle (port `Clock`), ni d'un service externe.
- Données de test : fictives et évidentes.

## 8. Dépendances

- Une dépendance nouvelle se justifie en une phrase dans la PR : besoin, alternative écartée, maintenance du paquet.
- **Délai de `MIN_RELEASE_AGE_DAYS` jours** (`.kit.conf`) : ne pas installer une version publiée depuis moins de ce délai (pnpm l'applique via `minimumReleaseAge`, Dependabot via `cooldown`, pour les autres écosystèmes on vérifie la date sur le registre). Les actions GitHub sont épinglées par SHA (`make pin-actions`).
- Une dépendance qui structure le projet (état, ORM, passerelle, cartographie) passe par un ADR.
- Les fichiers de verrouillage sont commités, jamais modifiés à la main.

## 9. Documentation et décisions

- Une décision d'architecture ou de produit devient un **ADR** (`tools/new-adr.sh`), pas un message de chat.
- Le vocabulaire est celui de `GLOSSARY.md` ; un terme nouveau y entre dans la même PR.
- Quand le code change une règle métier, le PRD ou le design correspondant change dans la **même PR**.
- Ce qui reste à trancher avec le client reste écrit dans la section « Questions ouvertes » du document concerné ; on ne le devine pas dans le code.
- Les affirmations vérifiées citent leur source ; ce qui n'a pas pu l'être est dit (« non vérifié »).
- [`docs/README.md`](docs/README.md) dit où est chaque document et quand le mettre à jour. Le modèle de PR a une case « Documentation » : à cocher, ou à remplacer par « aucun impact documentaire » et la raison.
- Diagrammes en Mermaid dans le Markdown ; le contrat d'API se génère ; les runbooks s'écrivent avec le premier besoin réel.

## 10. Relecture

Le relecteur contrôle, dans cet ordre : la sécurité et les invariants, le comportement (le test prouve-t-il la règle ?), la conception, la lisibilité. Les commentaires portent un niveau :

- **bloquant** : la PR ne se fusionne pas tant que ce n'est pas réglé ;
- **optionnel** (« nit ») : l'auteur décide, et dit en une ligne ce qu'il fait.

Un commentaire de sécurité ou sur un invariant n'est jamais optionnel. Un relecteur n'approuve pas ce qu'il n'a pas compris.

Un commentaire est **bloquant** seulement s'il porte sur :

- un invariant de `AGENTS.md` ;
- un défaut de sécurité ;
- une exigence explicite de la spec (PRD, ADR, design) qui concerne **cette** PR.

Ce que la spec prévoit pour une PR ultérieure (tests d'objets qui n'existent pas encore, règles à venir) est une **suggestion** : la revue reste un simple commentaire, sans « Changes requested ».

**Sources d'une revue** : le relecteur (humain ou agent) lit, avant de commenter, `AGENTS.md`, le `CODING.md` de chaque application touchée par la PR, `docs/policies/securite.md`, et les documents que la PR cite (PRD, ADR, design). Une revue qui ne s'appuie pas sur ces sources n'est pas une revue.

Avant de publier une revue, chaque constat est relu dans le document qu'il cite et donne le fichier et la section. Un constat non recoupé avec sa source ne se publie pas. Un test sur un nom de rôle, un constat « bloquant » qui porte sur ce que la spec prévoit pour plus tard, ou une affirmation non vérifiée sont les trois erreurs de revue les plus fréquentes.

## 11. Agents d'IA

Le code écrit par un agent suit les mêmes règles que celui d'un humain, et **la personne qui ouvre la PR en est responsable**. Un agent lit `AGENTS.md` avant de coder, ne contourne aucun hook, ne désactive aucun contrôle et ne fusionne rien sans demande explicite. Il ne signe rien : aucune mention d'outil ou d'agent dans les commits, descriptions de PR et commentaires. Les skills installées (`.agents/skills/`, versions figées par `skills-lock.json`, qu'on n'édite pas à la main) sont décrites dans `docs/guides/demarrage-developpeur.md`.

## 12. Sécurité

- Les règles techniques de sécurité par conception (SEC1 à SEC11) sont dans [`docs/policies/securite.md`](docs/policies/securite.md). Une PR qui y déroge le dit et cite la dérogation (§13).
- Une vulnérabilité ou une fuite de donnée **ne se signale pas dans une issue ou une PR publique** : prévenir directement le propriétaire du dépôt par un canal privé.
- Un secret publié par erreur est **considéré comme compromis** : il est révoqué et remplacé, retirer la ligne ne suffit pas.
- Les données d'identité ne quittent jamais les parcours prévus (lecture avec motif et audit).

## 13. Dérogation

Une règle de ce fichier ne s'écarte que par une décision écrite : un ADR, ou un commentaire de PR approuvé par le propriétaire du dépôt, daté, qui dit laquelle et pourquoi. Si la règle est fausse ou gênante, on la corrige ici, dans une PR.
