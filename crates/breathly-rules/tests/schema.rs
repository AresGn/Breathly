//! Schéma d'une règle : champs, langues, niveau de risque, formes, commande informative (US-060.2, US-024.2, TS-09).

mod common;

use breathly_rules::{
    Field, InformativeCommand, Language, Os, PathRoot, RiskLevel, RuleDefect, RuleRef, RulesError,
    Target, parse_cleanup_rules,
};
use common::{defect_of, rule_file};

#[test]
fn a_complete_folder_rule_is_accepted_with_its_fields() {
    let rules = parse_cleanup_rules(&rule_file(&[])).unwrap();

    let [rule] = rules.as_slice() else {
        panic!("une règle attendue")
    };
    assert_eq!(rule.id().as_str(), "npm-cache");
    assert_eq!(rule.name().en(), "npm cache");
    assert_eq!(
        rule.explain().fr(),
        "Paquets téléchargés par npm, retéléchargés si besoin."
    );
    assert_eq!(rule.risk(), RiskLevel::Safe);
    assert_eq!(rule.systems(), [Os::Linux, Os::Windows]);
    assert!(rule.command().is_none());
    let Target::Folder { paths } = rule.target() else {
        panic!("forme « folder » attendue")
    };
    let windows = &paths[&Os::Windows][0];
    assert_eq!(windows.root(), PathRoot::LocalAppData);
    assert_eq!(windows.components(), ["npm-cache", "_cacache"]);
}

#[test]
fn an_unknown_field_is_refused_and_named() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("delete_all", Some("true"))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::UnknownField {
            field: "delete_all".to_owned()
        }
    );
}

#[test]
fn an_unknown_field_inside_a_text_table_is_refused() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("name.de", Some(r#""npm""#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::UnknownField {
            field: "name.de".to_owned()
        }
    );
}

#[test]
fn an_incomplete_rule_is_refused_and_named() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("risk", None)])),
        "npm-cache",
    );
    assert_eq!(defect, RuleDefect::MissingField { field: Field::Risk });
}

#[test]
fn a_rule_without_identifier_is_named_by_its_position() {
    let error = parse_cleanup_rules(&rule_file(&[("id", None)])).unwrap_err();

    assert!(error.to_string().contains("n° 1"), "{error}");
    assert!(matches!(
        error,
        RulesError::InvalidRule {
            rule: RuleRef::Position(1),
            defect: RuleDefect::MissingField { field: Field::Id }
        }
    ));
}

#[test]
fn an_unknown_risk_level_is_refused_and_named() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("risk", Some(r#""low""#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::UnknownRiskLevel {
            value: "low".to_owned()
        }
    );
}

#[test]
fn a_missing_french_explanation_is_refused_and_named() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("explain.fr", None)])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::MissingText {
            field: Field::Explain,
            language: Language::Fr
        }
    );
}

#[test]
fn a_missing_english_explanation_is_refused_and_named() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("explain.en", None)])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::MissingText {
            field: Field::Explain,
            language: Language::En
        }
    );
}

#[test]
fn a_blank_explanation_counts_as_missing() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("explain.en", Some(r#""   ""#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::MissingText {
            field: Field::Explain,
            language: Language::En
        }
    );
}

#[test]
fn a_value_of_the_wrong_type_is_refused_and_named() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("risk", Some("3"))])),
        "npm-cache",
    );
    assert!(matches!(defect, RuleDefect::WrongType { .. }), "{defect:?}");
}

#[test]
fn an_invalid_identifier_is_refused() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("id", Some(r#""Npm Cache""#))])),
        "Npm Cache",
    );
    assert_eq!(
        defect,
        RuleDefect::InvalidId {
            value: "Npm Cache".to_owned()
        }
    );
}

#[test]
fn two_rules_with_the_same_identifier_are_refused() {
    let file = rule_file(&[]).repeat(2);
    let error = parse_cleanup_rules(&file).unwrap_err();

    assert!(error.to_string().contains("npm-cache"), "{error}");
    assert!(matches!(error, RulesError::DuplicateId { .. }));
}

#[test]
fn a_file_without_rule_or_with_a_stray_key_is_refused() {
    assert_eq!(parse_cleanup_rules(""), Err(RulesError::NoRule));
    let stray = format!("version = 2\n{}", rule_file(&[]));
    assert_eq!(
        parse_cleanup_rules(&stray),
        Err(RulesError::UnknownTopLevelKey {
            key: "version".to_owned()
        })
    );
    assert!(matches!(
        parse_cleanup_rules("[[rule"),
        Err(RulesError::Unreadable { .. })
    ));
}

#[test]
fn an_unknown_os_is_refused() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("os", Some(r#"["linux", "macos"]"#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::UnknownOs {
            value: "macos".to_owned()
        }
    );
}

#[test]
fn only_the_personal_decision_level_is_never_selectable() {
    assert!(RiskLevel::Safe.is_selectable());
    assert!(RiskLevel::Review.is_selectable());
    assert!(!RiskLevel::Personal.is_selectable());
}

#[test]
fn the_command_is_kept_for_display_and_never_run() {
    let workspace = tempfile::tempdir().unwrap();
    let sentinel = workspace.path().join("sentinel");
    let command = format!("touch {}", sentinel.display());
    let quoted = format!("{command:?}");
    let rules = parse_cleanup_rules(&rule_file(&[("command", Some(&quoted))])).unwrap();

    assert_eq!(
        rules[0].command().map(InformativeCommand::as_str),
        Some(command.as_str())
    );
    assert!(
        !sentinel.exists(),
        "lire une règle ne doit lancer aucun programme (TS-09)"
    );
}

#[test]
fn an_empty_command_is_refused() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("command", Some(r#""""#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::EmptyValue {
            field: Field::Command
        }
    );
}

#[test]
fn each_error_has_a_code_and_a_translation_key_built_from_it() {
    let errors = [
        parse_cleanup_rules("").unwrap_err(),
        parse_cleanup_rules(&rule_file(&[("risk", None)])).unwrap_err(),
    ];
    for error in errors {
        assert!(error.code().starts_with("rules."));
        assert_eq!(error.translation_key(), format!("errors.{}", error.code()));
    }
}
