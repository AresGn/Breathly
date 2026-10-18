# Maquettes de Breathly (V0.1)

Source de vérité visuelle de l'interface (`apps/desktop/ui`). Le contenu et les parcours des écrans sont décrits dans [`../0003-ecrans.md`](../0003-ecrans.md) ; ces fichiers montrent leur rendu. Un écran change dans la même PR que le code qui le modifie.

Origine : export de l'outil de maquettage Claude Design, livré le 2026-10-18 (`breathly-maquettes.zip`, non conservé dans le dépôt).

## Contenu

- **Direction visuelle « Respirer »** : calme, aéré, vert-bleu, vouvoiement, petite illustration d'accueil (`Main.dc.html`).
- **`breathly.css`** : les jetons de design (60 variables CSS, thèmes `data-theme="light"` et `"dark"`) et les composants. C'est la source du thème Svelte : **aucune couleur écrite en dur dans un composant**. `Jetons.dc.html` les nomme tous ; `Composants.dc.html` montre les composants en clair et en sombre.
- **Écrans**, un fichier chacun, clair et sombre, avec les états utiles (mode avancé, vide, chargement, erreur) :
  `H1`, `H2` (accueil) · `D1`, `D2`, `D3` (diagnostic) · `S1`, `S2`, `S3` (espace disque) · `N1` à `N5` (nettoyage) · `Y1`, `Y2`, `Y3` (historique) · `R1`, `R2`, `R3` (réglages) · `T1` (onglets, esquisse V0.3).
- **`Ecarts.dc.html`** : les 14 points ambigus relevés par le designer, les questions ouvertes du dossier, les parcours et les licences.
- **`logo/`** : la marque en SVG (clair, sombre, monochrome). Tuile arrondie vert-bleu, point central, anneau ouvert. **Tracé provisoire** : à vectoriser proprement, avec l'icône 512 px, avant la première publication.
- `canvas.json` : disposition des planches sur le canevas (outil de design seulement).

## Lire les fichiers `.dc.html`

Ce sont des planches du canevas de conception : elles s'affichent dans l'artefact Claude Design. Le script `support.js` que les pages référencent n'est **pas fourni** ; ouvertes directement dans un navigateur, elles n'affichent pas le rendu de référence. Pour le code, lire le HTML et le CSS comme référence de structure, de textes et de valeurs.

Les planches chargent des polices depuis Google Fonts **pour l'aperçu seulement**. L'application, elle, n'ouvre aucune connexion : **les polices sont embarquées en WOFF2** (ci-dessous).

## Décisions validées (2026-10-18)

Vouvoiement ; total libérable 13,7 Go avec les gros fichiers à part (8,4 Go) ; « Vider la corbeille » = case décochée dans S2, et si elle est cochée avec d'autres éléments, N1 puis la confirmation forte N5 (modifié le 2026-10-18 : le nettoyage a lieu d'abord, N5 s'affiche ensuite sur une liste recalculée, voir US-037.4) ; commande zram `sudo apt install zram-tools` (Debian, Ubuntu, Mint ; absente sous Fedora) ; seuils de R2 (espace libre 2 Go, disque occupé 90 %, swap 50 % si RAM disponible sous 15 %, température 80 °C au repos) ; pas de seuil de RAM ni de CPU en V0.1 ; le niveau de risque des recommandations réutilise les trois libellés (explication sans action : Sans risque, action de l'utilisateur sur le système : À vérifier) ; logo validé comme provisoire.

## Écarts connus et à traiter

1. **Libellés anglais** : les libellés de risque anglais ne sont pas dessinés. Valeurs retenues (reprises du README du dépôt) : **Safe**, **Review**, **Personal decision**.
2. **Texte à 200 %** : non maquetté. Comportement retenu : une seule colonne à la place des grilles à deux colonnes, défilement vertical (US-070.3, essai manuel à l'implémentation).
3. **Logo** : à vectoriser proprement, icône 512 px comprise (ticket avant la première publication).
4. **États d'historique** : « Incertain » est ajouté à l'écran Y1 ; les suppressions définitives s'affichent « Plus annulable » (`docs/design/0001-machines-a-etats.md`, §3).
5. **Anciens noyaux** : présents dans les données d'exemple mais volontairement absents de S1, S2 et N1 (droits administrateur, V0.4).
6. **Animations** : aucune maquettée ; seul l'anneau de H2 tourne, et seulement si `prefers-reduced-motion` n'est pas demandé. Pas de barre de titre dessinée (décorations natives différentes sous Windows et Linux).

## Polices et licences

Bricolage Grotesque, Hanken Grotesk et DM Mono sont publiées sous licence **OFL** : vérifié le 2026-10-18, chacune figure dans le dossier `ofl/` du dépôt `google/fonts` avec son fichier `OFL.txt` (DM Mono : `license: "OFL"` dans ses métadonnées). Le numéro de version de la licence (1.1 d'après le designer) n'a pas été relu. À embarquer en WOFF2 dans l'application, avec les fichiers de licence ; leur poids compte dans le budget de l'installeur (ADR 0008). Les icônes sont dessinées à la main dans l'esprit de Lucide (licence ISC, non vérifiée ici) ; le logo et l'illustration sont originaux.

## Mise à jour

Un changement visuel remplace le fichier concerné ici, dans la même PR que le code, avec la mention de l'écart si la maquette précède l'implémentation.
