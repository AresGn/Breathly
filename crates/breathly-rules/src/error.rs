//! Erreurs de `breathly-rules` : un code stable et une clé de traduction par variante (CODING.md §5).
//!
//! Une règle invalide est refusée avec un message qui la nomme (US-060.2). Les chemins cités sont ceux du fichier
//! de règles, jamais un chemin du disque de l'utilisateur (SEC9).

use std::fmt;

use thiserror::Error;

use crate::path::PathRefusal;
use crate::rule::{Os, RiskLevel, RuleId};

/// Code stable et clé de traduction qui s'en déduit (`errors.<code>`).
macro_rules! names {
    ($code:literal) => {
        ($code, concat!("errors.", $code))
    };
}

/// Désignation d'une règle dans un message : son identifiant tel qu'écrit, ou sa position quand il manque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleRef {
    Id(String),
    Position(usize),
}

impl fmt::Display for RuleRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => write!(f, "« {id} »"),
            Self::Position(position) => write!(f, "n° {position} (sans identifiant)"),
        }
    }
}

/// Champ d'une règle, tel qu'écrit dans le fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Id,
    Name,
    Explain,
    Risk,
    Os,
    Kind,
    Paths,
    Folder,
    Marker,
    Extensions,
    Command,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Explain => "explain",
            Self::Risk => "risk",
            Self::Os => "os",
            Self::Kind => "kind",
            Self::Paths => "paths",
            Self::Folder => "folder",
            Self::Marker => "marker",
            Self::Extensions => "extensions",
            Self::Command => "command",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Fr,
    En,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Fr => "fr",
            Self::En => "en",
        })
    }
}

/// Ce qui rend une règle invalide.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleDefect {
    #[error("champ inconnu « {field} »")]
    UnknownField { field: String },
    #[error("champ obligatoire absent : « {field} »")]
    MissingField { field: Field },
    #[error("texte « {field}.{language} » absent ou vide")]
    MissingText { field: Field, language: Language },
    #[error("valeur de mauvais type : {detail}")]
    WrongType { detail: String },
    #[error(
        "identifiant invalide « {value} » (minuscules, chiffres et tirets, 64 caractères au plus)"
    )]
    InvalidId { value: String },
    #[error("niveau de risque inconnu « {value} » (safe, review ou personal)")]
    UnknownRiskLevel { value: String },
    #[error("système inconnu « {value} » (linux ou windows)")]
    UnknownOs { value: String },
    #[error(
        "forme inconnue « {value} » (folder, marked-folder, files-by-extension ou system-trash)"
    )]
    UnknownKind { value: String },
    #[error("valeur en double dans « {field} »")]
    DuplicateValue { field: Field },
    #[error("champ « {field} » sans objet pour cette forme de règle")]
    FieldNotAllowedForKind { field: Field },
    #[error("aucun chemin pour le système « {os} » (paths.{os})")]
    MissingPathsForOs { os: Os },
    #[error("chemins pour « {os} », absent de la liste os")]
    PathsForUnlistedOs { os: Os },
    #[error("la corbeille du système n'accepte aucun chemin")]
    PathsForSystemTrash,
    #[error("chemin refusé « {path} » (paths.{os}) : {refusal}")]
    InvalidPath {
        os: Os,
        path: String,
        refusal: PathRefusal,
    },
    #[error("nom refusé « {value} » dans « {field} » : {refusal}")]
    InvalidName {
        field: Field,
        value: String,
        refusal: PathRefusal,
    },
    #[error(
        "extension refusée « {value} » (lettres et chiffres, sans point ; jamais un conteneur chiffré)"
    )]
    InvalidExtension { value: String },
    #[error(
        "niveau « {risk} » interdit pour cette forme : safe est réservé à la forme folder (S3)"
    )]
    RiskNotAllowedForKind { risk: RiskLevel },
    #[error("champ « {field} » vide")]
    EmptyValue { field: Field },
}

impl RuleDefect {
    const fn names(&self) -> (&'static str, &'static str) {
        match self {
            Self::UnknownField { .. } => names!("rules.unknown_field"),
            Self::MissingField { .. } => names!("rules.missing_field"),
            Self::MissingText { .. } => names!("rules.missing_text"),
            Self::WrongType { .. } => names!("rules.wrong_type"),
            Self::InvalidId { .. } => names!("rules.invalid_id"),
            Self::UnknownRiskLevel { .. } => names!("rules.unknown_risk_level"),
            Self::UnknownOs { .. } => names!("rules.unknown_os"),
            Self::UnknownKind { .. } => names!("rules.unknown_kind"),
            Self::DuplicateValue { .. } => names!("rules.duplicate_value"),
            Self::FieldNotAllowedForKind { .. } => names!("rules.field_not_allowed_for_kind"),
            Self::MissingPathsForOs { .. } => names!("rules.missing_paths_for_os"),
            Self::PathsForUnlistedOs { .. } => names!("rules.paths_for_unlisted_os"),
            Self::PathsForSystemTrash => names!("rules.paths_for_system_trash"),
            Self::InvalidPath { .. } => names!("rules.invalid_path"),
            Self::InvalidName { .. } => names!("rules.invalid_name"),
            Self::InvalidExtension { .. } => names!("rules.invalid_extension"),
            Self::RiskNotAllowedForKind { .. } => names!("rules.risk_not_allowed_for_kind"),
            Self::EmptyValue { .. } => names!("rules.empty_value"),
        }
    }
}

/// Fichier de règles refusé. Une seule erreur à la fois : la première rencontrée.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RulesError {
    #[error("fichier de règles illisible : {detail}")]
    Unreadable { detail: String },
    #[error("clé inconnue « {key} » hors d'une règle")]
    UnknownTopLevelKey { key: String },
    #[error("aucune règle `[[rule]]` dans le fichier")]
    NoRule,
    #[error("« rule » doit être une liste de tables `[[rule]]`")]
    RuleNotAList,
    #[error("identifiant en double : « {} »", id.as_str())]
    DuplicateId { id: RuleId },
    #[error("fichier de règles « {file} » : {error}")]
    InvalidFile {
        file: String,
        error: Box<RulesError>,
    },
    #[error("règle {rule} : {defect}")]
    InvalidRule { rule: RuleRef, defect: RuleDefect },
}

impl RulesError {
    /// Code stable, repris dans les journaux et par l'interface.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.names().0
    }

    /// Clé du texte affiché, dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        self.names().1
    }

    const fn names(&self) -> (&'static str, &'static str) {
        match self {
            Self::Unreadable { .. } => names!("rules.unreadable"),
            Self::UnknownTopLevelKey { .. } => names!("rules.unknown_top_level_key"),
            Self::NoRule => names!("rules.no_rule"),
            Self::RuleNotAList => names!("rules.rule_not_a_list"),
            Self::DuplicateId { .. } => names!("rules.duplicate_id"),
            Self::InvalidFile { error, .. } => error.names(),
            Self::InvalidRule { defect, .. } => defect.names(),
        }
    }
}
