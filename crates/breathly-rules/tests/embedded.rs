//! Règles du dépôt intégrées au binaire (US-060.1, US-060.3, ADR 0004).

use std::fs;
use std::path::Path;

use breathly_rules::{RiskLevel, Target, embedded_rule_file_names, load_cleanup_rules};

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

/// Chaque fichier `*.toml` de `rules/` est intégré : aucun n'est laissé de côté par le build.
#[test]
fn every_toml_file_of_the_rules_folder_is_embedded() {
    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules");
    let mut on_disk: Vec<String> = fs::read_dir(rules_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.to_ascii_lowercase().ends_with(".toml"))
        .collect();
    on_disk.sort();

    assert_eq!(embedded_rule_file_names().collect::<Vec<_>>(), on_disk);
}

/// Jetons qui ouvriraient un fichier ou liraient l'environnement à l'exécution.
const FORBIDDEN_AT_RUN_TIME: [&str; 9] = [
    "std::fs",
    "fs::",
    "File::",
    "OpenOptions",
    "read_dir",
    "read_to_string",
    "include_bytes!",
    "std::env",
    "env::var",
];

/// À l'exécution, aucun dossier de règles n'est lu (ADR 0004, SEC6) : le code du crate, sous-dossiers compris et
/// hors commentaires, n'ouvre aucun fichier. Seul `build.rs`, exécuté à la compilation, lit `rules/`.
#[test]
fn the_crate_reads_no_file_at_run_time() {
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                assert_reads_no_file(&path);
            }
        }
    }
}

fn assert_reads_no_file(path: &Path) {
    let Ok(code) = fs::read_to_string(path) else {
        panic!("{} illisible", path.display());
    };
    let lines = code
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"));
    for line in lines {
        let found = FORBIDDEN_AT_RUN_TIME
            .iter()
            .find(|token| line.contains(*token));
        assert!(found.is_none(), "{} : « {line} »", path.display());
    }
}
