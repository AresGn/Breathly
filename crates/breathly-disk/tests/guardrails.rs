//! TS-01 : emplacements de règle, niveau de risque, liens, réglages (SEC2). Tout se passe dans un dossier temporaire
//! avec les faux de `breathly-platform` : aucun test ne touche au vrai dossier personnel ni à la vraie corbeille.

use std::fs;

use breathly_disk::{Candidate, Guardrails, Refusal};
use breathly_platform::StandardPaths;
use breathly_platform::fake::link_directory;
use breathly_rules::RiskLevel;

mod common;
use common::{Fixture, TestResult, refused_in_home, rule};

#[test]
fn an_item_inside_a_rule_location_is_verified() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/old.bin")?;
    let rule = rule("safe")?;
    let roots = [fixture.home().join(".cache/demo")];
    let candidate = Candidate {
        rule: &rule,
        roots: &roots,
        path: &item,
    };
    let verified = Guardrails::new(&fixture.paths, &fixture.open_files).verify(&candidate)?;
    assert_eq!(verified.rule().as_str(), "demo-cache");
    assert_eq!(verified.risk(), RiskLevel::Safe);
    assert_eq!(verified.anchor().file_name(), "old.bin");
    assert!(verified.origin().ends_with("old.bin"));
    assert!(!format!("{verified:?}").contains("old.bin"));
    Ok(())
}

#[test]
fn a_review_level_item_is_verified_too() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/old.bin")?;
    let roots = [fixture.home().join(".cache/demo")];
    assert_eq!(fixture.verdict(&rule("review")?, (&roots, &item)), Ok(()));
    Ok(())
}

#[test]
fn a_red_item_is_never_selectable() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/old.bin")?;
    let roots = [fixture.home().join(".cache/demo")];
    assert_eq!(
        fixture.verdict(&rule("personal")?, (&roots, &item)),
        Err(Refusal::NotSelectable)
    );
    Ok(())
}

#[test]
fn an_item_outside_every_rule_location_is_refused() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file("notes/todo.txt")?;
    let roots = [fixture.home().join(".cache/demo")];
    fs::create_dir_all(&roots[0])?;
    assert_eq!(
        fixture.verdict(&rule("safe")?, (&roots, &item)),
        Err(Refusal::OutsideRule)
    );
    Ok(())
}

#[test]
fn a_rule_location_is_never_an_item_itself() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.file(".cache/demo/old.bin")?;
    let root = fixture.home().join(".cache/demo");
    assert_eq!(
        fixture.verdict(&rule("safe")?, (std::slice::from_ref(&root), &root)),
        Err(Refusal::OutsideRule)
    );
    Ok(())
}

#[test]
fn a_link_to_a_protected_folder_is_not_followed() -> TestResult {
    let fixture = Fixture::new()?;
    let precious = fixture.file("Documents/rapport.odt")?;
    let cache = fixture.home().join(".cache").join("demo");
    fs::create_dir_all(&cache)?;
    link_directory(&cache.join("shortcut"), &fixture.home().join("Documents"))?;
    let refusal = fixture
        .verdict(
            &rule("safe")?,
            (
                std::slice::from_ref(&cache),
                &cache.join("shortcut/rapport.odt"),
            ),
        )
        .err()
        .ok_or("l'élément derrière le lien aurait dû être refusé")?;
    assert!(matches!(
        refusal,
        Refusal::OutsideRule | Refusal::UserFolder
    ));
    assert!(precious.exists());
    Ok(())
}

#[test]
fn a_missing_item_is_refused_without_error() -> TestResult {
    let fixture = Fixture::new()?;
    let root = fixture.home().join(".cache/demo");
    fs::create_dir_all(&root)?;
    assert_eq!(
        fixture.verdict(
            &rule("safe")?,
            (std::slice::from_ref(&root), &root.join("gone.bin"))
        ),
        Err(Refusal::PathUnresolvable)
    );
    Ok(())
}

#[test]
fn no_settings_file_disables_a_guardrail() -> TestResult {
    let fixture = Fixture::new()?;
    let config = fixture.paths.config_dir()?;
    fs::create_dir_all(&config)?;
    for forged in [
        r#"{"guardrails": false, "disable_guardrails": true, "allow_documents": true}"#,
        "guardrails = \"off\"\nprotect_user_folders = false\n",
    ] {
        fs::write(config.join("settings.json"), forged)?;
        fs::write(config.join("settings.toml"), forged)?;
        let refusal = refused_in_home(&fixture, "Documents/rapport.odt")?;
        assert_eq!(refusal, Refusal::UserFolder);
    }
    Ok(())
}

#[test]
fn every_refusal_has_a_code_and_a_translation_key_built_from_it() {
    let all = [
        Refusal::NotSelectable,
        Refusal::OutsideRule,
        Refusal::UserFolder,
        Refusal::BrowserProfile,
        Refusal::EncryptedContainer,
        Refusal::GitFolder,
        Refusal::PathUnresolvable,
        Refusal::PathChanged,
        Refusal::PathBusy,
        Refusal::PathDenied,
    ];
    for refusal in all {
        assert!(refusal.code().starts_with("guard."));
        assert_eq!(
            refusal.translation_key(),
            format!("errors.{}", refusal.code())
        );
    }
}
