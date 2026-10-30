//! Règles intégrées au binaire : le contenu des fichiers `rules/*.toml`, copié à la compilation par `build.rs`.

use crate::error::RulesError;
use crate::rule::Rule;
use crate::{parse_cleanup_rules, reject_duplicate_ids};

/// (nom du fichier dans `rules/`, contenu), triés par nom.
const EMBEDDED_RULE_FILES: &[(&str, &str)] =
    include!(concat!(env!("OUT_DIR"), "/embedded_rules.rs"));

/// Règles de nettoyage livrées avec Breathly, validées. Un identifiant ne peut apparaître qu'une fois, tous
/// fichiers confondus.
///
/// # Errors
///
/// [`RulesError::InvalidFile`] qui nomme le fichier et la règle fautifs, ou [`RulesError::DuplicateId`]. Les tests
/// du crate chargent ces règles : un fichier invalide fait échouer `make check` avant d'atteindre un binaire.
pub fn load_cleanup_rules() -> Result<Vec<Rule>, RulesError> {
    let mut rules = Vec::new();
    for &(file, source) in EMBEDDED_RULE_FILES {
        let parsed = parse_cleanup_rules(source).map_err(|error| RulesError::InvalidFile {
            file: file.to_owned(),
            error: Box::new(error),
        })?;
        rules.extend(parsed);
    }
    reject_duplicate_ids(&rules)?;
    Ok(rules)
}
