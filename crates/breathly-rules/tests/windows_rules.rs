//! Règles de départ propres à Windows (`rules/windows.toml`, ticket #54, PRD §8).

use breathly_rules::{Os, PathRoot, RiskLevel, Rule, Target, load_cleanup_rules};

fn embedded_rule(id: &str) -> Rule {
    load_cleanup_rules()
        .unwrap_or_else(|error| panic!("{error}"))
        .into_iter()
        .find(|rule| rule.id().as_str() == id)
        .unwrap_or_else(|| panic!("règle « {id} » absente de rules/"))
}

fn windows_folder(rule: &Rule) -> (PathRoot, Vec<String>) {
    let Target::Folder { paths } = rule.target() else {
        panic!("forme « folder » attendue pour « {} »", rule.id().as_str())
    };
    let [path] = paths[&Os::Windows].as_slice() else {
        panic!(
            "un seul chemin Windows attendu pour « {} »",
            rule.id().as_str()
        )
    };
    (path.root(), path.components().to_vec())
}

#[test]
fn the_temp_rule_targets_the_user_temp_folder() {
    let rule = embedded_rule("windows-temp");

    assert_eq!(rule.systems(), [Os::Windows]);
    assert_eq!(rule.risk(), RiskLevel::Safe);
    assert_eq!(windows_folder(&rule), (PathRoot::Temp, Vec::new()));
}

#[test]
fn the_crash_dumps_rule_targets_the_default_dump_folder() {
    let rule = embedded_rule("windows-crash-dumps");

    assert_eq!(rule.systems(), [Os::Windows]);
    assert_eq!(rule.risk(), RiskLevel::Review);
    assert_eq!(
        windows_folder(&rule),
        (PathRoot::LocalAppData, vec!["CrashDumps".to_owned()])
    );
}
