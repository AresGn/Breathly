# Polices embarquées

Fichiers WOFF2 servis par l'application elle-même (aucune connexion, SEC1). Sources : dépôt `google/fonts`, dossier `ofl/`, relevé le 2026-11-08. Licence : SIL Open Font License 1.1, relue le 2026-11-08 dans chaque fichier `OFL-*.txt`.

| Fichier | Famille | Graisses | Taille |
|---|---|---|---|
| `BricolageGrotesque-Variable.woff2` | Bricolage Grotesque | 500 à 700 (opsz 14, wdth 100 figés) | 46 676 o |
| `HankenGrotesk-Variable.woff2` | Hanken Grotesk | 400 à 700 | 26 012 o |
| `DMMono-Regular.woff2` | DM Mono | 400 | 16 472 o |
| `DMMono-Medium.woff2` | DM Mono | 500 | 16 692 o |

Total : 105 852 o (environ 103 Kio). Sous-ensemble : latin et latin étendu (français, anglais), ponctuation, flèches.

Regénération : `fonttools varLib.instancer` pour figer les axes inutiles, puis `pyftsubset --flavor=woff2` avec les plages Unicode ci-dessus.
