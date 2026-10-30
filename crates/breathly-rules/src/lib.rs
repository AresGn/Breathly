//! Module `rules` : format et validation des règles de nettoyage (TOML intégré au binaire, ADR 0004).
//!
//! Les fichiers `rules/*.toml` du dépôt sont intégrés à la compilation (`build.rs`) : aucun dossier de règles
//! n'est lu à l'exécution. Format : `docs/design/0004-format-des-regles-de-nettoyage.md`. Le champ `command`
//! est lu et conservé pour l'affichage en mode avancé ; ce crate ne lance jamais de programme (SEC6).

mod embedded;
mod error;
mod path;
mod raw;
mod rule;
mod validate;

use std::collections::BTreeSet;

pub use embedded::{embedded_rule_file_names, load_cleanup_rules};
pub use error::{Field, Language, RuleDefect, RuleRef, RulesError};
pub use path::{PathRefusal, PathRoot, RulePath};
pub use rule::{InformativeCommand, LocalizedText, Os, PathsByOs, RiskLevel, Rule, RuleId, Target};

use crate::raw::RawRule;

/// Lit et valide un fichier de règles (`[[rule]]`). La première règle invalide refuse tout le fichier.
///
/// # Errors
///
/// [`RulesError`] : fichier illisible, clé hors règle, aucune règle, identifiant en double, ou règle invalide
/// (nommée par son identifiant ou sa position).
pub fn parse_cleanup_rules(source: &str) -> Result<Vec<Rule>, RulesError> {
    let mut document: toml::Table =
        source
            .parse()
            .map_err(|error: toml::de::Error| RulesError::Unreadable {
                detail: error.message().to_owned(),
            })?;
    if let Some(key) = document.keys().find(|key| *key != "rule") {
        return Err(RulesError::UnknownTopLevelKey { key: key.clone() });
    }
    let entries = match document.remove("rule") {
        Some(toml::Value::Array(entries)) if !entries.is_empty() => entries,
        Some(toml::Value::Array(_)) | None => return Err(RulesError::NoRule),
        Some(_) => return Err(RulesError::RuleNotAList),
    };
    let rules = entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| parse_rule(index + 1, entry))
        .collect::<Result<Vec<_>, _>>()?;
    reject_duplicate_ids(&rules)?;
    Ok(rules)
}

fn parse_rule(position: usize, entry: toml::Value) -> Result<Rule, RulesError> {
    let rule = match entry.get("id").and_then(toml::Value::as_str) {
        Some(id) => RuleRef::Id(id.to_owned()),
        None => RuleRef::Position(position),
    };
    let invalid = |defect| RulesError::InvalidRule {
        rule: rule.clone(),
        defect,
    };
    let raw: RawRule = entry.try_into().map_err(|error: toml::de::Error| {
        invalid(RuleDefect::WrongType {
            detail: error.message().to_owned(),
        })
    })?;
    validate::validate(raw).map_err(invalid)
}

fn reject_duplicate_ids(rules: &[Rule]) -> Result<(), RulesError> {
    let mut seen = BTreeSet::new();
    match rules.iter().find(|rule| !seen.insert(rule.id())) {
        Some(rule) => Err(RulesError::DuplicateId {
            id: rule.id().clone(),
        }),
        None => Ok(()),
    }
}
