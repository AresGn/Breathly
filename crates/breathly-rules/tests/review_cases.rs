//! Cas relevés en revue : `%TEMP%` seul, alias Windows, niveau de risque selon la forme, textes des messages.

mod common;

use breathly_rules::{Field, PathRefusal, RiskLevel, RuleDefect, RulesError, parse_cleanup_rules};
use common::{defect_of, rule_file};

fn windows_refusal(path: &str) -> Option<PathRefusal> {
    let paths = format!("[{path:?}]");
    let file = rule_file(&[
        ("os", Some(r#"["windows"]"#)),
        ("paths.linux", None),
        ("paths.windows", Some(&paths)),
    ]);
    match parse_cleanup_rules(&file) {
        Ok(_) => None,
        Err(error) => {
            let defect = defect_of(Err(error), "npm-cache");
            let RuleDefect::InvalidPath { refusal, .. } = defect else {
                panic!("chemin « {path} » : refus de chemin attendu, obtenu {defect:?}");
            };
            Some(refusal)
        }
    }
}

#[test]
fn the_temporary_folder_may_be_cleaned_whole_but_no_other_root() {
    assert_eq!(windows_refusal("%TEMP%"), None);
    for root in ["%LOCALAPPDATA%", "%APPDATA%", "%USERPROFILE%"] {
        assert_eq!(
            windows_refusal(root),
            Some(PathRefusal::RootItself),
            "{root}"
        );
    }
}

#[test]
fn names_that_windows_trims_are_refused() {
    for path in [
        "%USERPROFILE%/Documents.",
        "%USERPROFILE%/Documents ",
        "%LOCALAPPDATA%/projet/.git.",
    ] {
        assert_eq!(
            windows_refusal(path),
            Some(PathRefusal::ForbiddenCharacter),
            "{path}"
        );
    }
}

#[test]
fn safe_is_reserved_to_whole_folder_rules() {
    let file = rule_file(&[
        ("kind", Some(r#""system-trash""#)),
        ("paths.linux", None),
        ("paths.windows", None),
    ]);
    let defect = defect_of(parse_cleanup_rules(&file), "npm-cache");
    assert_eq!(
        defect,
        RuleDefect::RiskNotAllowedForKind {
            risk: RiskLevel::Safe
        }
    );
}

#[test]
fn messages_use_the_toml_names_of_fields_and_systems() {
    let error = parse_cleanup_rules(&rule_file(&[("explain.en", None)])).unwrap_err();
    assert_eq!(
        error.to_string(),
        "règle « npm-cache » : texte « explain.en » absent ou vide"
    );
    let error =
        parse_cleanup_rules(&rule_file(&[("paths.windows", Some(r#"["~/x"]"#))])).unwrap_err();
    assert_eq!(
        error.to_string(),
        "règle « npm-cache » : chemin refusé « ~/x » (paths.windows) : racine d'un autre système"
    );
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("risk", None)])),
        "npm-cache",
    );
    assert_eq!(defect, RuleDefect::MissingField { field: Field::Risk });
    assert_eq!(defect.to_string(), "champ obligatoire absent : « risk »");
}

#[test]
fn a_rule_key_that_is_not_a_list_of_tables_is_named_as_such() {
    for file in ["[rule]\nid = \"npm-cache\"\n", "rule = \"npm-cache\"\n"] {
        assert_eq!(
            parse_cleanup_rules(file),
            Err(RulesError::RuleNotAList),
            "{file}"
        );
    }
}
