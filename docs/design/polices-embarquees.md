# Polices embarquées

Fichiers WOFF2 servis par l'application elle-même (aucune connexion, SEC1). Les sources sont épinglées à une révision du dépôt `google/fonts` par famille. Licence : SIL Open Font License 1.1, relue le 2026-11-08 dans chaque fichier `OFL-*.txt`.

| Famille | Version | Révision de `google/fonts` |
|---|---|---|
| Bricolage Grotesque | 1.001 | [`b9f6c712`](https://github.com/google/fonts/commit/b9f6c712059d72742282ebdf06eadfc264c827f3) |
| Hanken Grotesk | 3.013 | [`71489156`](https://github.com/google/fonts/commit/714891563e901b1a0d8ebcaaa003b01604793888) |
| DM Mono | 1.000 | [`85f32a02`](https://github.com/google/fonts/commit/85f32a0254c400b643d7ce589b91a2ad29bea73b) |

Les fichiers WOFF2 dérivent des fichiers de ces révisions, sous-ensemblés pour Breathly. Les licences OFL correspondantes restent dans `apps/desktop/ui/static/fonts/`.

| Fichier | Famille | Graisses | Taille |
|---|---|---|---|
| `BricolageGrotesque-Variable.woff2` | Bricolage Grotesque | 500 à 700 (axes optique et largeur figés) | 46 676 o |
| `HankenGrotesk-Variable.woff2` | Hanken Grotesk | 400 à 700 | 26 012 o |
| `DMMono-Regular.woff2` | DM Mono | 400 | 16 472 o |
| `DMMono-Medium.woff2` | DM Mono | 500 | 16 692 o |

Total WOFF2 : 105 852 o (environ 103 Kio). Les fichiers contiennent `œ`, `«`, `»`, `’`, U+00A0 et `€`. U+202F est présent dans Bricolage Grotesque, mais absent des fichiers sources Hanken Grotesk et DM Mono à ces révisions ; pour ces deux familles, le navigateur utilise la police de repli du système pour ce caractère.

Le test borne le poids des WOFF2 à 150 Kio : environ 1 % du budget d'installeur de 15 Mo (ADR 0008), avec environ 47 Kio de marge sur le poids actuel. Ce plafond porte uniquement sur les fichiers WOFF2, pas sur les licences ni les autres ressources.

## Taille des installeurs

La mesure de 2,78 Mio de l'ADR 0008 porte sur la page vide, avant ces polices. Les `.deb` et `.rpm` avec les polices n'ont pas été mesurés dans cette PR : cet environnement Windows ne dispose pas d'un outil de construction Linux. Le `.msi` reste également à mesurer lors d'une construction Windows, comme prévu par l'ADR 0008. Le poids brut des quatre WOFF2 (105 852 o) est documenté ci-dessus, mais ne remplace pas une mesure des paquets construits.

## Régénération

Utiliser les sources épinglées ci-dessus, `fonttools varLib.instancer` pour figer les axes inutiles, puis `pyftsubset --flavor=woff2` avec les plages Unicode conservées dans les fichiers actuels et ajouter U+202F si la source le contient. Vérifier les fichiers produits et leurs tailles avant de les remplacer.
