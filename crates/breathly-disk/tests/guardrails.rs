//! TS-01 : un test par garde-fou de S5 (SEC2). Tout se passe dans un dossier temporaire avec les faux de
//! `breathly-platform` : aucun test ne touche au vrai dossier personnel ni à la vraie corbeille.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use breathly_disk::{Candidate, Guardrails, Refusal};
use breathly_platform::StandardPaths;
use breathly_platform::fake::{FixedStandardPaths, link_directory};
use breathly_rules::{RiskLevel, Rule, parse_cleanup_rules};
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn Error>>;

struct Fixture {
    sandbox: TempDir,
    paths: FixedStandardPaths,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn Error>> {
        let sandbox = tempfile::tempdir()?;
        fs::create_dir_all(sandbox.path().join("home"))?;
        let paths = FixedStandardPaths::under(sandbox.path());
        Ok(Self { sandbox, paths })
    }

    fn home(&self) -> PathBuf {
        self.sandbox.path().join("home")
    }

    fn file(&self, relative: &str) -> Result<PathBuf, Box<dyn Error>> {
        let path = self.home().join(relative);
        fs::create_dir_all(path.parent().ok_or("chemin sans parent")?)?;
        fs::write(&path, b"contenu")?;
        Ok(path)
    }

    fn verdict(&self, rule: &Rule, place: (&[PathBuf], &Path)) -> Result<(), Refusal> {
        let (roots, path) = place;
        verdict_of(&self.paths, Candidate { rule, roots, path })
    }
}

fn verdict_of(paths: &FixedStandardPaths, candidate: Candidate<'_>) -> Result<(), Refusal> {
    Guardrails::new(paths).verify(&candidate).map(drop)
}

fn rule(risk: &str) -> Result<Rule, Box<dyn Error>> {
    let source = format!(
        r#"[[rule]]
id = "demo-cache"
name.fr = "Cache de démonstration"
name.en = "Demo cache"
explain.fr = "Fichiers recréés automatiquement."
explain.en = "Files that are re-created automatically."
risk = "{risk}"
os = ["linux", "windows"]
kind = "folder"
paths.linux = ["~/.cache/demo"]
paths.windows = ["%LOCALAPPDATA%/demo"]
"#
    );
    Ok(parse_cleanup_rules(&source)?.remove(0))
}

fn refused_in_home(fixture: &Fixture, relative: &str) -> Result<Refusal, Box<dyn Error>> {
    let item = fixture.file(relative)?;
    let roots = [fixture.home()];
    fixture
        .verdict(&rule("safe")?, (&roots, &item))
        .err()
        .ok_or_else(|| format!("{relative} aurait dû être refusé").into())
}

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
    let verified = Guardrails::new(&fixture.paths).verify(&candidate)?;
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
fn the_five_user_folders_are_refused_by_their_standard_names() -> TestResult {
    let fixture = Fixture::new()?;
    for name in ["Documents", "Pictures", "Videos", "Music", "Desktop"] {
        let refusal = refused_in_home(&fixture, &format!("{name}/photo.jpg"))?;
        assert_eq!(refusal, Refusal::UserFolder, "{name}");
    }
    Ok(())
}

#[test]
fn the_user_folders_are_refused_by_their_french_names_too() -> TestResult {
    let fixture = Fixture::new()?;
    for name in ["Images", "Vidéos", "Musique", "Bureau"] {
        let refusal = refused_in_home(&fixture, &format!("{name}/fichier.dat"))?;
        assert_eq!(refusal, Refusal::UserFolder, "{name}");
    }
    Ok(())
}

#[test]
fn an_ancestor_of_a_user_folder_is_refused_too() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.file("Documents/rapport.odt")?;
    let roots = [fixture.sandbox.path().to_path_buf()];
    assert_eq!(
        fixture.verdict(&rule("safe")?, (&roots, &fixture.home())),
        Err(Refusal::UserFolder)
    );
    Ok(())
}

#[test]
fn the_browser_profile_locations_are_refused() -> TestResult {
    let fixture = Fixture::new()?;
    for location in [
        ".mozilla/firefox/abcd.default/places.sqlite",
        ".config/google-chrome/Default/Cookies",
        ".config/chromium/Default/Cache/data_0",
        ".config/BraveSoftware/Brave-Browser/Default/History",
        "AppData/Roaming/Mozilla/Firefox/Profiles/abcd/key4.db",
        "AppData/Local/Google/Chrome/User Data/Default/Cache/f_000001",
        "AppData/Local/Microsoft/Edge/User Data/Default/Login Data",
    ] {
        let refusal = refused_in_home(&fixture, location)?;
        assert_eq!(refusal, Refusal::BrowserProfile, "{location}");
    }
    Ok(())
}

#[test]
fn a_folder_that_contains_a_browser_profile_is_refused() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.file(".mozilla/firefox/abcd.default/places.sqlite")?;
    let roots = [fixture.home()];
    assert_eq!(
        fixture.verdict(&rule("safe")?, (&roots, &fixture.home().join(".mozilla"))),
        Err(Refusal::BrowserProfile)
    );
    Ok(())
}

#[test]
fn encrypted_containers_are_refused_whatever_the_case() -> TestResult {
    let fixture = Fixture::new()?;
    for name in ["vault.hc", "disk.vc", "passwords.kdbx", "PASSWORDS.KDBX"] {
        let refusal = refused_in_home(&fixture, &format!(".cache/demo/{name}"))?;
        assert_eq!(refusal, Refusal::EncryptedContainer, "{name}");
    }
    Ok(())
}

#[test]
fn a_folder_that_holds_an_encrypted_container_is_refused() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.file(".cache/demo/deep/er/vault.vc")?;
    let roots = [fixture.home().join(".cache")];
    assert_eq!(
        fixture.verdict(
            &rule("safe")?,
            (&roots, &fixture.home().join(".cache/demo"))
        ),
        Err(Refusal::EncryptedContainer)
    );
    Ok(())
}

#[test]
fn a_git_folder_is_refused_and_so_is_what_holds_one() -> TestResult {
    let fixture = Fixture::new()?;
    let inside = refused_in_home(&fixture, ".cache/demo/project/.git/HEAD")?;
    assert_eq!(inside, Refusal::GitFolder);
    let roots = [fixture.home().join(".cache")];
    assert_eq!(
        fixture.verdict(
            &rule("safe")?,
            (&roots, &fixture.home().join(".cache/demo/project"))
        ),
        Err(Refusal::GitFolder)
    );
    Ok(())
}

#[test]
fn a_link_to_a_protected_folder_is_not_followed() -> TestResult {
    let fixture = Fixture::new()?;
    let precious = fixture.file("Documents/rapport.odt")?;
    let cache = fixture.home().join(".cache/demo");
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
    ];
    for refusal in all {
        assert!(refusal.code().starts_with("guard."));
        assert_eq!(
            refusal.translation_key(),
            format!("errors.{}", refusal.code())
        );
    }
}
