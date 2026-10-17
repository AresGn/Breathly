# Principes de conception (indépendants du langage)

Ce guide est le **« pourquoi »** des règles de `AGENTS.md`. Les exemples sont en pseudo-code ; la traduction exacte (outil, syntaxe, seuil contrôlé par la machine) est dans le `CODING.md` de chaque pile.

## 1. Pourquoi ces règles

Un code lu vingt fois pour une écriture doit se comprendre sans l'auteur. Chaque règle vise l'un de ces buts : **rendre un état illégal impossible à représenter**, **rendre un oubli visible à la compilation**, **garder une règle métier à un seul endroit**.

## 2. SOLID, appliqué au quotidien

- **Responsabilité unique** : si la description d'une fonction contient « et » ou « puis », ce sont deux fonctions.
- **Ouvert/fermé** : un nouveau cas s'ajoute (nouvelle variante d'énumération, nouvel adapter), il ne modifie pas dix `if` existants. Une énumération avec traitement exhaustif fait de l'oubli une erreur de compilation.
- **Substitution** : un adapter remplaçable respecte le contrat de son port, y compris ses erreurs.
- **Ségrégation des interfaces** : des interfaces petites, déclarées là où on les consomme.
- **Inversion de dépendance** : le domaine déclare le port ; l'infrastructure fournit l'adapter ; la composition les branche. Le domaine n'importe jamais un framework.

## 3. DRY, KISS, YAGNI — et leurs limites

- **DRY** porte sur une *règle métier*, pas sur deux morceaux de code qui se ressemblent. Une règle écrite à deux endroits est un défaut ; deux boucles voisines ne le sont pas.
- **KISS** : la solution la plus simple qui satisfait la règle et le test.
- **YAGNI** : pas de paramètre, d'interface, de fabrique ou de hook pour un besoin qui n'existe pas encore. **Règle des 3** : trois cas semblables justifient d'extraire, deux non. Exception documentée : un *seam* hypothétique assumé (un port posé pour que la V2 y branche un vrai adapter).

## 4. Types : le compilateur travaille pour vous

- Interdire le type flou (`any`, `mixed`, `dynamic`, `interface{}`) hors des frontières ; **valider et typer tout de suite** ce qui entre (requête, JSON, variable d'environnement, colonne).
- **Un type par concept** : un identifiant de ressource n'est pas un identifiant d'utilisateur, un montant n'est pas un entier. Deux paramètres de même type brut s'inversent sans erreur.
- **Énumération + traitement exhaustif** pour tout état et toute catégorie ; pas de cas par défaut, pour que l'ajout d'une variante casse l'analyse là où elle manque.
- **Union discriminée** pour un état d'écran ou de calcul (chargement, erreur, donnée) plutôt que des booléens en cascade.
- **Immutabilité par défaut** : une transition retourne un nouvel état.

```
// À refuser                          // Attendu
send(userId, itemId)                  send(UserId, ItemId)   // inversion = erreur de compilation
status: string                        status: Status         // enum fermé, traitement exhaustif
```

## 5. Taille et forme

| Mesure | Seuil | Action |
|---|---|---|
| Lignes d'une fonction | > 40 | extraire des sous-fonctions nommées |
| Complexité cyclomatique | > 10 | simplifier ou découper |
| Imbrication | > 3 | retour anticipé, extraction |
| Paramètres positionnels | > 3 | objet de commande ou paramètres nommés |
| Lignes d'un fichier | > 250 à 300 | un fichier = un concept |

**Garde d'abord, chemin nominal en dernier** : chaque refus sort par un retour anticipé, le corps ne s'indente pas pour le cas normal.

## 6. Nommage

Le métier d'abord, avec le vocabulaire de `GLOSSARY.md` : jamais `data`, `manager`, `helper`, `process`, `thing`. Booléens : `is`, `has`, `can`, `should`. Fonctions : un verbe d'action. Pas d'abréviation obscure. Un booléen muet en paramètre (`publish(item, true)`) devient deux fonctions ou un type nommé.

## 7. Erreurs

- **Échec métier attendu** (permission refusée, transition interdite, ressource introuvable) : **valeur de retour typée** (énumération de raison ou résultat). Ce n'est pas une exception.
- **Invariant violé / défaut de programmation** : exception (ou `panic`). Elle remonte, est journalisée avec son contexte et la réponse ne détaille rien.
- **Ne jamais avaler une erreur** : pas de capture vide. On attrape pour ajouter du contexte puis relancer, ou pour décider explicitement (commentaire qui dit pourquoi).
- Un message d'erreur et un journal ne contiennent aucune donnée personnelle.

## 8. Testabilité

- Écrire le code testable par construction : logique métier **pure** (sans I/O), effets (horloge, hasard, réseau, fichiers) derrière des ports injectés.
- **Pyramide** : beaucoup de tests de domaine rapides, des tests d'API en boîte noire contre une vraie base pour les parcours, peu de tests de bout en bout.
- On teste le **comportement observable**, jamais l'implémentation. Un bug se corrige en écrivant d'abord le test qui le reproduit.
- On ne simule pas ce qu'on possède mal (la base de données, une classe finale) : on utilise la vraie base, on simule l'interface.
- Un test qui échoue ne se saute pas, ne s'affaiblit pas et ne se supprime pas pour passer au vert.

## 9. Commentaires

Un commentaire dit le **pourquoi** qu'aucun nom ne porte (contrainte du PRD, piège d'une bibliothèque, décision d'ADR). Un commentaire qui répète la ligne suivante est supprimé. Les mises à jour d'un document de décision se datent.

## 10. Code smells — détection rapide

| Smell | Symptôme | Remède |
|---|---|---|
| Fonction « Dieu » | fait tout, > 40 lignes, plusieurs responsabilités | responsabilité unique, extraction |
| Chirurgie à fusil | un changement métier modifie 8 fichiers | centraliser la règle dans le domaine |
| Envie des fonctionnalités | une fonction manipule surtout les données d'un autre module | la rapprocher des données |
| Obsession des primitives | `string` partout au lieu de types métier | objets-valeurs ou types dédiés |
| Cécité booléenne | `update(true, false, true)` illisible à l'appel | objet nommé ou union |
| Chaîne de caractères typée | `status: string` au lieu d'une énumération | énumération fermée |
| Modèle anémique | entités = sacs de données, logique éparpillée ailleurs | rapprocher comportement et données |
| Enfer des callbacks | chaînes imbriquées | async/await ou équivalent |
| Intermédiaire pur | une classe qui ne fait que déléguer | l'appelant appelle la cible |
| Abstraction spéculative | interface, paramètre ou hook sans besoin | supprimer jusqu'au vrai besoin |

## 11. Checklist de relecture personnelle (avant de pousser)

- [ ] Chaque fonction fait une seule chose (test du « et / puis »)
- [ ] Aucun type flou sans justification écrite
- [ ] Toute union d'états est une énumération, tout traitement est exhaustif
- [ ] Aucune fonction > 40 lignes, aucun nesting > 3, aucun > 3 paramètres positionnels
- [ ] Toute erreur est soit journalisée soit relancée, jamais avalée
- [ ] Les erreurs métier attendues sont des valeurs de retour
- [ ] Aucun commentaire qui répète le code
- [ ] La logique métier pure est testable sans simuler d'I/O
- [ ] Aucune règle métier dupliquée, aucune abstraction sans besoin
