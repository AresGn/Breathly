# Documentation du domaine

Comment les skills d'ingénierie lisent la documentation du domaine de ce dépôt (un seul contexte).

## À lire avant d'explorer

- **`GLOSSARY.md`** à la racine.
- **`docs/adr/`** : les ADR qui touchent la zone sur laquelle on va travailler.
- Pour un ticket, les documents qu'il cite (`docs/prd/`, `docs/design/`, le `CODING.md` de l'application, `docs/policies/securite.md`). L'index est `docs/README.md`.

## Disposition

```
/
├── GLOSSARY.md
├── docs/
│   ├── adr/         décisions d'architecture
│   ├── prd/         exigences
│   ├── design/      conception (système, base, écrans)
│   ├── policies/    sécurité et conformité
│   └── agents/      configuration des skills (ce dossier)
└── <applications déclarées dans .kit.conf>
```

## Employer le vocabulaire du glossaire

Un titre de ticket, un nom de test ou une proposition emploie les termes tels que `GLOSSARY.md` les définit, sans synonyme que le glossaire écarte. Un concept absent du glossaire est soit du vocabulaire inventé (à reconsidérer), soit un vrai manque (à signaler).

## Signaler un conflit avec un ADR

Si une sortie contredit un ADR, le dire explicitement plutôt que de le contourner :

> _Contredit l'ADR 0007 (…), mais mérite d'être rouvert parce que…_
