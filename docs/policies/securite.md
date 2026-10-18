# Sécurité par conception

- **Statut** : règles proposées pour Breathly (étape 2.8 du parcours) ; à adapter par ADR (jamais à affaiblir en silence)
- **Date** : 2026-10-18
- **Portée** : tout le code de Breathly (cœur Rust, interface Svelte, extension, installeurs). Les invariants sont dans `AGENTS.md` ; ce document ajoute les règles techniques qui les protègent.
- **Lecteurs** : toute personne ou tout agent qui écrit du code.
- **Numérotation** : les règles s'appellent **SEC1 à SEC11** pour ne pas se confondre avec les exigences S1 à S6 du PRD (disque).

**Principe** : une règle n'existe que si elle a un garant, une vérification automatique ou une ligne de relecture. Chaque ligne ci-dessous dit lequel. Breathly agit sur les fichiers de personnes non techniques : **l'interface web est traitée comme non fiable** (elle peut être compromise ou forgée), et tout fichier local est validé au chargement.

## 1. Règles en vigueur

| # | Règle | Garant |
|---|---|---|
| SEC1 | **Aucune connexion réseau en V0.1.** Aucune bibliothèque réseau (client HTTP, TLS) dans l'arbre de dépendances du cœur et de l'interface. Pas de compte, pas de télémétrie. | `cargo deny check bans` sur l'arbre de dépendances (`deny.toml`), prouvé par sonde négative le 2026-10-18 (étape 4) ; test d'absence de connexion (US-071) |
| SEC2 | **Suppression : seulement par une règle, jamais hors des garde-fous.** Le chemin est résolu et normalisé, les liens symboliques et jonctions ne sont pas suivis, les garde-fous S5 du PRD sont **revérifiés pour chaque élément au moment d'agir**. | un test par garde-fou, tests de liens symboliques et de jonctions, relecture |
| SEC3 | **L'interface n'envoie jamais un chemin.** Elle envoie des identifiants d'éléments de la dernière analyse et un jeton de confirmation lié à une simulation, à usage unique. Chaque commande Tauri est enregistrée explicitement et restreinte à la fenêtre principale. | test sur l'inventaire des commandes, test du jeton, relecture |
| SEC4 | **Historique avant action.** Une entrée « pending » est écrite avant chaque action ; si l'historique n'est pas inscriptible, aucune suppression n'a lieu. L'interface est l'unique écrivain. | tests du design 0001 (invariant 2) |
| SEC5 | **Fichiers locaux non fiables.** Réglages, liste d'exclusion et historique sont validés au chargement (type, bornes, taille maximale) ; une valeur invalide est refusée avec un message, jamais corrigée en silence. La liste d'exclusion ne peut que **restreindre** un nettoyage ; aucun fichier ne peut désactiver un garde-fou. Les fichiers sont créés avec des droits limités à l'utilisateur. | tests de chargement avec fichiers forgés, relecture |
| SEC6 | **Règles intégrées au binaire.** Aucune règle n'est lue depuis un dossier modifiable à l'exécution (V0.1). Le champ `command` n'est jamais exécuté. Une règle nouvelle est relue par **deux personnes** (PRD §16). | validation en CI (ADR 0004), relecture |
| SEC7 | **Élévation (V0.4) par un assistant à liste fermée.** Breathly ne tourne jamais en administrateur, ne manipule jamais un mot de passe et ne passe jamais une commande libre ; l'assistant revalide chaque paramètre contre ses propres règles. | tests d'arguments forgés, relecture (ADR 0006) |
| SEC8 | **Communication locale (V0.2-V0.3) sans port réseau.** Socket de domaine Unix ou tube nommé aux droits de l'utilisateur ; messages JSON versionnés, validés, de taille bornée (1 Mo au plus vers le navigateur). L'hôte de messagerie native n'accepte que l'extension déclarée (liste exacte, sans joker). | test de connexion d'un autre compte, test de messages forgés |
| SEC9 | **Confidentialité des journaux.** On ne journalise que des identifiants et des codes, jamais le contenu d'un fichier ni un chemin complet hors de l'historique local. Les URL ne sortent jamais du navigateur : l'extension ne transmet que le domaine et des compteurs (PRD T3). | test qui cherche les données interdites dans la sortie, relecture |
| SEC10 | **Dépendances maîtrisées.** `cargo audit` en CI ; mises à jour hebdomadaires par Dependabot avec délai de 7 jours ; toute nouvelle dépendance se demande d'abord (PRD §14) ; actions de CI épinglées par SHA. | CI, Dependabot, `tools/pin-actions.py` |
| SEC11 | **Distribution vérifiable.** Chaque release publie les empreintes SHA-256 de tous les installeurs, générées par la CI, avec la procédure de vérification dans le README. Signature de code à la V1.0. Installeurs construits depuis un tag par la CI. | étape de release de la CI, relecture |

## 2. Valeurs de départ

À valider par mesure ou par ADR ; elles se changent sans modifier la règle.

| Paramètre | Valeur proposée |
|---|---|
| Taille maximale d'un fichier de réglages à la lecture | 1 Mo |
| Taille maximale d'un message de l'hôte vers le navigateur | 1 Mo (limite des navigateurs, vérifiée) |
| Conservation de l'historique | 12 mois (ADR 0003) |
| Délai avant mise à jour d'une dépendance publiée | 7 jours |

## 3. Décisions à prendre par un ADR avant le premier code sensible

- **Suppression sûre** : stratégie contre l'écart entre vérification et action. **Le prototype P1 (2026-10-18) a démontré la faille** : si un dossier du chemin devient un lien symbolique entre la vérification et l'action, `trash::delete` suit le lien et atteint un fichier protégé ; recanoniser juste avant d'agir la détecte sans fermer la fenêtre. À décider : comparaison d'identité du dossier parent avant et après, ou opérations relatives à un descripteur pour la suppression définitive. Normalisation sous Windows (chemins longs, jonctions) non étudiée.
- **Configuration Tauri** : politique de contenu (CSP) de l'interface, liste des capacités, interdiction de tout contenu distant. À lire dans la documentation de Tauri v2 avant d'écrire la configuration.
- **Garde réseau** : outil exact de contrôle de l'arbre de dépendances (SEC1).
- **Signature de code** (V1.0) : certificat Windows, clé de signature Linux, stockage des secrets de signature dans la CI.
- **Attestation de provenance des builds** : à lire dans la documentation de GitHub avant tout choix.

## 4. Pistes écartées

| Piste | Raison |
|---|---|
| Signer l'historique et les réglages | la clé serait lisible par le même compte que le logiciel malveillant visé : complexité sans gain |
| Mode « superadmin » (Breathly entier en administrateur) | toute faille deviendrait une suppression sans limite |
| `pkexec` d'une commande libre | pkexec ne valide pas les arguments (man pkexec) |
| Règles lues depuis un dossier modifiable | un logiciel malveillant pourrait y déposer une règle (ADR 0004) |
| Exécuter le champ `command` des règles | programme tiers lancé, simulation et historique non fiables (ADR 0005, US-060.4) |
| Port TCP local pour le service | accessible à tout processus de la machine (ADR 0007) |
| Télémétrie ou vérification de version automatique avant la V1.0 | contredit « aucune donnée ne sort » |

## 5. Sources vérifiées

| Affirmation | Source (lue le 2026-10-18) |
|---|---|
| pkexec ne valide pas les arguments ; une action polkit peut être liée à un chemin | page de manuel de pkexec |
| Native messaging : JSON, longueur sur 32 bits, 1 Mo de l'hôte vers le navigateur ; Chrome liste les origines autorisées sans joker | documentation MDN et Chrome |
| Les contenus d'une page web ne peuvent pas appeler l'hôte directement (Chrome) ; l'extension valide `sender.origin` | documentation Chrome |
| Tauri v2 : par défaut toute commande enregistrée est appelable depuis toute fenêtre | documentation Tauri, « Capabilities » |
| Crate `trash` 5.2.9 : `purge_all`, `restore_all` sur Windows et Linux | docs.rs ; essai P1 sous Linux le 2026-10-18 (liens symboliques, collision de restauration, purge sélective) |

**Non vérifié** : la fermeture complète de la fenêtre entre vérification et action pour `trash`, le comportement d'UAC, le fonctionnement exact de `cargo audit` et de l'outil de garde réseau, l'attestation de provenance GitHub, la gestion des liens symboliques et jonctions par les bibliothèques retenues, la CSP de Tauri.
