//! Lecture brute d'une règle : tous les champs sont facultatifs et les clés inconnues sont gardées, pour que la
//! validation nomme précisément ce qui manque ou ce qui est en trop (au lieu du message générique de `serde`).

use std::collections::BTreeMap;

use serde::Deserialize;

pub(crate) type Unknown = BTreeMap<String, toml::Value>;

#[derive(Debug, Deserialize)]
pub(crate) struct RawRule {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<RawText>,
    pub(crate) explain: Option<RawText>,
    pub(crate) risk: Option<String>,
    pub(crate) os: Option<Vec<String>>,
    pub(crate) kind: Option<String>,
    pub(crate) paths: Option<RawPaths>,
    pub(crate) folder: Option<String>,
    pub(crate) marker: Option<String>,
    pub(crate) extensions: Option<Vec<String>>,
    pub(crate) command: Option<String>,
    #[serde(flatten)]
    pub(crate) unknown: Unknown,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawText {
    pub(crate) fr: Option<String>,
    pub(crate) en: Option<String>,
    #[serde(flatten)]
    pub(crate) unknown: Unknown,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawPaths {
    pub(crate) linux: Option<Vec<String>>,
    pub(crate) windows: Option<Vec<String>>,
    #[serde(flatten)]
    pub(crate) unknown: Unknown,
}

impl RawRule {
    /// Première clé inconnue, préfixée par sa table (`name.de`), dans l'ordre alphabétique.
    pub(crate) fn first_unknown_field(&self) -> Option<String> {
        let nested = |table: &str, unknown: Option<&Unknown>| {
            unknown
                .and_then(|keys| keys.keys().next())
                .map(|key| format!("{table}.{key}"))
        };
        self.unknown
            .keys()
            .next()
            .cloned()
            .or_else(|| nested("name", self.name.as_ref().map(|t| &t.unknown)))
            .or_else(|| nested("explain", self.explain.as_ref().map(|t| &t.unknown)))
            .or_else(|| nested("paths", self.paths.as_ref().map(|p| &p.unknown)))
    }
}
