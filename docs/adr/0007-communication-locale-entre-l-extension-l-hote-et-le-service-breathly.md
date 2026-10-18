# ADR 0007 : Communication locale entre l'extension, l'hôte et le service Breathly

- **Statut** : proposé
- **Date** : 2026-10-18
- **Décideurs** : propriétaire du dépôt (AresGn)

## Contexte

L'extension (V0.3) ne peut parler qu'à un hôte de messagerie native lancé par le navigateur. Le service Breathly (V0.2) tourne séparément. Il faut un lien local sûr et léger, sans port réseau ouvert (confidentialité, G5).

Vérifié (documentation MDN et Chrome, lues le 2026-10-18) : les messages sont du JSON UTF-8 précédé d'une longueur sur 32 bits **en ordre natif des octets**, sur stdin et stdout de l'hôte ; de l'hôte vers le navigateur, **1 Mo au plus** ; du navigateur vers l'hôte, **4 Go** (Firefox) et **64 MiB** (Chrome) ; `connectNative` garde l'hôte lancé tant que le port est ouvert, `sendNativeMessage` lance un processus par message ; Firefox référence le manifeste par clé de registre sous Windows ; Chrome liste les extensions autorisées dans `allowed_origins` (pas de joker). La page de Chrome ne parle pas des navigateurs en bac à sable (snap, flatpak) : **ce point reste le risque du PRD §16**, à lever par le prototype 2.9.

## Options pour le lien hôte, interface et service

1. **Socket locale** (socket de domaine Unix sous Linux, tube nommé sous Windows), droits limités à l'utilisateur (retenu).
2. TCP sur 127.0.0.1 : accessible à tout processus, secret nécessaire.
3. Le service lance l'hôte à chaque message : contredit B1 (2 s) et G5.

## Décision

Option 1. Messages JSON avec un champ de version, un type et un identifiant de requête, comme le native messaging, pour que l'hôte relaie sans transformer. L'extension utilise `connectNative` (hôte persistant). L'interface Tauri parle au service par le même transport. Le crate qui implémente socket et tube est un choix d'implémentation ; en ajouter un se demande d'abord (PRD §14).

## Conséquences

- Aucun port réseau n'est ouvert ; le service refuse les connexions d'un autre utilisateur.
- Les messages de l'hôte vers le navigateur restent bien sous 1 Mo (statistiques et décisions de veille, pas de listes d'onglets complètes). Les URL ne sortent jamais du navigateur : seuls le domaine et des compteurs transitent (T3).
- Le champ de version permet de faire évoluer le protocole sans casser une extension plus ancienne.
- Le prototype 2.9 (P3) a vérifié, sous Linux : démarrage de l'hôte par **Chromium 147** et **Firefox 150**, relais vers une socket locale en 0600, tailles de messages. À vérifier encore : Zen, LibreWolf, Brave, Chrome, Edge, Windows, navigateurs en snap et en Flatpak, plan B du PRD (mode autonome de l'extension).

## Résultats du prototype P3 (2026-10-18)

Détail dans `docs/research/2026-10-18-prototypes-2-9.md`.

- **Confirmé** : Chromium et Firefox lancent l'hôte, lui passent l'origine de l'extension (Chromium) ou le chemin du manifeste puis l'identifiant (Firefox). Du navigateur vers l'hôte, 2 Mo passent sur Chromium. De l'hôte vers le navigateur, 500 Ko passent et **1,2 Mo est refusé par Chromium** : le protocole garde donc les messages de l'hôte nettement sous 1 Mo.
- **Manifeste par profil (Chromium)** : l'hôte est cherché dans le dossier du profil du navigateur ; l'installation doit écrire le manifeste pour chaque navigateur et profil visés, ou à l'emplacement système.
- **Risque confirmé pour les Flatpak** : un navigateur en Flatpak comme Zen n'a, par défaut, ni accès au dossier personnel ni la possibilité de lancer un programme de l'hôte ; un hôte installé sur le système ne peut pas y être lancé. Non exécuté, déduit des permissions. Plan B : mode autonome de l'extension, ou hôte fourni dans le bac à sable, à étudier.
