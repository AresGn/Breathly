# Politique de sécurité

## Signaler une vulnérabilité

**Ne publiez pas de faille dans une issue ou une PR publique.** Utilisez le signalement privé de GitHub :
<https://github.com/AresGn/Breathly/security/advisories/new> (onglet *Security* du dépôt, bouton *Report a vulnerability*).

À REMPLIR: activer « Private vulnerability reporting » dans Settings > Code security du dépôt (sans cela, le lien ci-dessus ne fonctionne pas), puis supprimer cette ligne. Si le signalement privé n'est pas disponible, indiquer ici une adresse de contact dédiée.

Indiquez : la version concernée, le système d'exploitation, les étapes pour reproduire, l'effet constaté et, si possible, la correction envisagée.

## Ce que vous pouvez attendre de nous

- un accusé de réception sous **7 jours** ;
- une première évaluation (confirmée, refusée, besoin d'information) sous **14 jours** ;
- une correction publiée avant toute divulgation, puis un remerciement dans les notes de version si vous le souhaitez.

Ces délais sont des objectifs d'un projet maintenu bénévolement, pas des garanties.

## Versions couvertes

Seule la **dernière version publiée** reçoit des correctifs de sécurité. À REMPLIR: préciser si les versions précédentes sont couvertes.

## Périmètre

Une faille est ce qui permet à un tiers de lire, modifier ou supprimer ce que l'utilisateur n'a pas autorisé, d'exécuter du code, ou de contourner un garde-fou documenté dans [`AGENTS.md`](AGENTS.md). Un bug fonctionnel sans effet de sécurité se signale dans une issue normale.

## Pour les contributeurs

Les règles techniques de sécurité du dépôt sont dans [`docs/policies/securite.md`](docs/policies/securite.md). Ne jamais commiter de secret : `make check` exécute un scan de secrets.
