//! Passage d'une règle brute à une `Rule` validée. Aucune valeur n'est corrigée en silence (SEC5) : tout écart
//! est refusé.

use std::collections::BTreeSet;

use crate::error::{Field, Language, RuleDefect};
use crate::path::{Placement, check_name, parse_rule_path};
use crate::raw::{RawPaths, RawRule, RawText};
use crate::rule::{
    InformativeCommand, LocalizedText, Os, PathsByOs, RiskLevel, Rule, RuleId, Target,
};

/// Extensions de conteneurs chiffrés, de coffres et de clés, jamais nettoyables (S5, liste « … » élargie).
const ENCRYPTED_CONTAINER_EXTENSIONS: [&str; 10] = [
    "hc", "vc", "tc", "kdbx", "kdb", "luks", "gpg", "pgp", "age", "key",
];
const EXTENSION_MAX_LEN: usize = 16;

/// Forme d'une règle, telle qu'écrite dans `kind`.
#[derive(Debug, Clone, Copy)]
enum Kind {
    Folder,
    MarkedFolder,
    FilesByExtension,
    SystemTrash,
}

impl Kind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "folder" => Some(Self::Folder),
            "marked-folder" => Some(Self::MarkedFolder),
            "files-by-extension" => Some(Self::FilesByExtension),
            "system-trash" => Some(Self::SystemTrash),
            _ => None,
        }
    }

    /// Champs propres à une forme (`folder`, `marker`, `extensions`) que cette forme accepte.
    fn accepts(self, field: Field) -> bool {
        match self {
            Self::Folder | Self::SystemTrash => false,
            Self::MarkedFolder => matches!(field, Field::Folder | Field::Marker),
            Self::FilesByExtension => matches!(field, Field::Extensions),
        }
    }
}

/// Champs propres à la forme de la règle.
struct TargetFields {
    paths: Option<RawPaths>,
    folder: Option<String>,
    marker: Option<String>,
    extensions: Option<Vec<String>>,
}

pub(crate) fn validate(raw: RawRule) -> Result<Rule, RuleDefect> {
    if let Some(field) = raw.first_unknown_field() {
        return Err(RuleDefect::UnknownField { field });
    }
    let id = RuleId::try_from(required(raw.id, Field::Id)?)?;
    let name = localized(raw.name, Field::Name)?;
    let explain = localized(raw.explain, Field::Explain)?;
    let risk = required(raw.risk, Field::Risk)?;
    let risk = RiskLevel::from_key(&risk).ok_or(RuleDefect::UnknownRiskLevel { value: risk })?;
    let systems = systems(required(raw.os, Field::Os)?)?;
    let kind = required(raw.kind, Field::Kind)?;
    let kind = Kind::parse(&kind).ok_or(RuleDefect::UnknownKind { value: kind })?;
    check_risk_for_kind(risk, kind)?;
    let fields = TargetFields {
        paths: raw.paths,
        folder: raw.folder,
        marker: raw.marker,
        extensions: raw.extensions,
    };
    let target = target(kind, fields, &systems)?;
    let command = raw.command.map(command).transpose()?;
    Ok(Rule {
        id,
        name,
        explain,
        risk,
        systems,
        target,
        command,
    })
}

fn required<T>(value: Option<T>, field: Field) -> Result<T, RuleDefect> {
    value.ok_or(RuleDefect::MissingField { field })
}

fn localized(raw: Option<RawText>, field: Field) -> Result<LocalizedText, RuleDefect> {
    let raw = required(raw, field)?;
    let text = |value: Option<String>, language| {
        value
            .filter(|text| !text.trim().is_empty())
            .ok_or(RuleDefect::MissingText { field, language })
    };
    Ok(LocalizedText::new(
        text(raw.fr, Language::Fr)?,
        text(raw.en, Language::En)?,
    ))
}

/// 🟢 seulement pour un dossier entier (un cache) : le reste est 🟡 au moins (GLOSSARY, PRD §8), et seuls les
/// caches 🟢 se suppriment définitivement (S3).
fn check_risk_for_kind(risk: RiskLevel, kind: Kind) -> Result<(), RuleDefect> {
    let is_allowed = match risk {
        RiskLevel::Safe => matches!(kind, Kind::Folder),
        RiskLevel::Review | RiskLevel::Personal => true,
    };
    if is_allowed {
        Ok(())
    } else {
        Err(RuleDefect::RiskNotAllowedForKind { risk })
    }
}

fn systems(raw: Vec<String>) -> Result<Vec<Os>, RuleDefect> {
    let systems = raw
        .into_iter()
        .map(|value| Os::from_key(&value).ok_or(RuleDefect::UnknownOs { value }))
        .collect::<Result<Vec<_>, _>>()?;
    if systems.is_empty() {
        return Err(RuleDefect::EmptyValue { field: Field::Os });
    }
    reject_duplicates(&systems, Field::Os)?;
    Ok(systems)
}

fn target(kind: Kind, fields: TargetFields, systems: &[Os]) -> Result<Target, RuleDefect> {
    reject_foreign_fields(kind, &fields)?;
    match kind {
        Kind::Folder => {
            let paths = paths_by_os(fields.paths, systems, Placement::Cleaned)?;
            Ok(Target::Folder { paths })
        }
        Kind::MarkedFolder => {
            let folder = name(required(fields.folder, Field::Folder)?, Field::Folder)?;
            let marker = name(required(fields.marker, Field::Marker)?, Field::Marker)?;
            let roots = paths_by_os(fields.paths, systems, Placement::SearchRoot)?;
            Ok(Target::MarkedFolder {
                folder,
                marker,
                roots,
            })
        }
        Kind::FilesByExtension => {
            let extensions = extensions(required(fields.extensions, Field::Extensions)?)?;
            let folders = paths_by_os(fields.paths, systems, Placement::Cleaned)?;
            Ok(Target::FilesByExtension {
                extensions,
                folders,
            })
        }
        Kind::SystemTrash => Ok(Target::SystemTrash),
    }
}

/// Refuse un champ qui n'a pas de sens pour la forme de la règle, au lieu de l'ignorer.
fn reject_foreign_fields(kind: Kind, fields: &TargetFields) -> Result<(), RuleDefect> {
    if matches!(kind, Kind::SystemTrash) && fields.paths.is_some() {
        return Err(RuleDefect::PathsForSystemTrash);
    }
    let present = [
        (fields.folder.is_some(), Field::Folder),
        (fields.marker.is_some(), Field::Marker),
        (fields.extensions.is_some(), Field::Extensions),
    ];
    match present
        .into_iter()
        .find(|&(is_present, field)| is_present && !kind.accepts(field))
    {
        Some((_, field)) => Err(RuleDefect::FieldNotAllowedForKind { field }),
        None => Ok(()),
    }
}

fn paths_by_os(
    raw: Option<RawPaths>,
    systems: &[Os],
    placement: Placement,
) -> Result<PathsByOs, RuleDefect> {
    let raw = required(raw, Field::Paths)?;
    let mut by_os = PathsByOs::new();
    for (os, listed) in [(Os::Linux, raw.linux), (Os::Windows, raw.windows)] {
        match (systems.contains(&os), listed) {
            (true, Some(list)) if !list.is_empty() => {
                by_os.insert(os, paths_for(os, list, placement)?);
            }
            (true, None | Some(_)) => return Err(RuleDefect::MissingPathsForOs { os }),
            (false, Some(_)) => return Err(RuleDefect::PathsForUnlistedOs { os }),
            (false, None) => {}
        }
    }
    Ok(by_os)
}

fn paths_for(
    os: Os,
    list: Vec<String>,
    placement: Placement,
) -> Result<Vec<crate::path::RulePath>, RuleDefect> {
    let paths = list
        .into_iter()
        .map(|path| match parse_rule_path(&path, os, placement) {
            Ok(parsed) => Ok(parsed),
            Err(refusal) => Err(RuleDefect::InvalidPath { os, path, refusal }),
        })
        .collect::<Result<Vec<_>, _>>()?;
    reject_duplicates(&paths, Field::Paths)?;
    Ok(paths)
}

fn name(value: String, field: Field) -> Result<String, RuleDefect> {
    match check_name(&value) {
        Ok(()) => Ok(value),
        Err(refusal) => Err(RuleDefect::InvalidName {
            field,
            value,
            refusal,
        }),
    }
}

fn extensions(raw: Vec<String>) -> Result<Vec<String>, RuleDefect> {
    if raw.is_empty() {
        return Err(RuleDefect::EmptyValue {
            field: Field::Extensions,
        });
    }
    let is_valid = |value: &str| {
        (1..=EXTENSION_MAX_LEN).contains(&value.len())
            && value
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            && !ENCRYPTED_CONTAINER_EXTENSIONS.contains(&value)
    };
    if let Some(value) = raw.iter().find(|value| !is_valid(value)) {
        return Err(RuleDefect::InvalidExtension {
            value: value.clone(),
        });
    }
    reject_duplicates(&raw, Field::Extensions)?;
    Ok(raw)
}

fn command(text: String) -> Result<InformativeCommand, RuleDefect> {
    if text.trim().is_empty() {
        return Err(RuleDefect::EmptyValue {
            field: Field::Command,
        });
    }
    Ok(InformativeCommand::new(text))
}

fn reject_duplicates<T: Ord>(values: &[T], field: Field) -> Result<(), RuleDefect> {
    let distinct: BTreeSet<&T> = values.iter().collect();
    if distinct.len() == values.len() {
        Ok(())
    } else {
        Err(RuleDefect::DuplicateValue { field })
    }
}
