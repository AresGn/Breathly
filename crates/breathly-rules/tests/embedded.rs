//! Règles du dépôt intégrées au binaire (US-060.1, US-060.3, ADR 0004).

use std::fs;
use std::path::Path;

use breathly_rules::{RiskLevel, Target, load_cleanup_rules};

/// Charge chaque fichier de `rules/` : une règle invalide (explication manquante, chemin hors dossier autorisé…)
/// fait échouer ce test, donc `make check` et la CI, avec un message qui nomme le fichier et la règle.
#[test]
fn every_rule_of_the_repository_is_valid() {
    let rules = load_cleanup_rules().unwrap_or_else(|error| panic!("{error}"));
    assert!(!rules.is_empty(), "rules/ ne contient aucune règle");
}

#[test]
fn the_npm_cache_rule_is_embedded() {
    let rules = load_cleanup_rules().unwrap();
    let npm = rules
        .iter()
        .find(|rule| rule.id().as_str() == "npm-cache")
        .expect("règle npm-cache absente de rules/");

    assert_eq!(npm.risk(), RiskLevel::Safe);
    assert!(matches!(npm.target(), Target::Folder { .. }));
    assert!(!npm.explain().fr().is_empty() && !npm.explain().en().is_empty());
}

/// À l'exécution, aucun dossier de règles n'est lu (ADR 0004, SEC6) : le code du crate n'ouvre aucun fichier.
/// Seul `build.rs`, exécuté à la compilation, lit `rules/`.
#[test]
fn the_crate_reads_no_file_at_run_time() {
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in fs::read_dir(&sources).unwrap() {
        let path = entry.unwrap().path();
        let code = fs::read_to_string(&path).unwrap();
        for forbidden in [
            "std::fs",
            "File::",
            "read_dir",
            "read_to_string",
            "std::env",
        ] {
            assert!(
                !code.contains(forbidden),
                "{} contient « {forbidden} »",
                path.display()
            );
        }
    }
}
