//! TS-01 (suite) : dossiers de l'utilisateur, profils de navigateur, conteneurs chiffrés, `.git`.

use breathly_disk::Refusal;

mod common;
use common::{Fixture, TestResult, refused_in_home, rule};

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
        ".config/mozilla/firefox/abcd.default/places.sqlite",
        ".var/app/com.google.Chrome/config/google-chrome/Default/Cookies",
        ".var/app/org.chromium.Chromium/config/chromium/Default/Cookies",
        ".var/app/com.brave.Browser/config/BraveSoftware/Brave-Browser/Default/History",
        ".var/app/com.microsoft.Edge/config/microsoft-edge/Default/History",
        ".config/vivaldi/Default/History",
        ".config/opera/Default/History",
        "AppData/Local/Packages/Mozilla.Firefox_n80bbvh6b1yt8/LocalCache/Roaming/Mozilla/Firefox/Profiles/a/key4.db",
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
    for name in [
        "vault.hc",
        "disk.vc",
        "disk.tc",
        "passwords.kdbx",
        "PASSWORDS.KDBX",
        "old.kdb",
        "disk.luks",
        "secrets.gpg",
        "mail.pgp",
        "backup.age",
        "id.key",
    ] {
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
fn a_user_folder_the_system_cannot_name_makes_the_item_refused_not_allowed() -> TestResult {
    let fixture = Fixture::new()?;
    let item = fixture.file(".cache/demo/old.bin")?;
    let roots = [fixture.home().join(".cache/demo")];
    let paths = common::PathsWithoutVideos(&fixture.paths);
    let candidate = breathly_disk::Candidate {
        rule: &rule("safe")?,
        roots: &roots,
        path: &item,
    };
    let verdict = breathly_disk::Guardrails::new(&paths, &fixture.open_files).verify(&candidate);
    assert_eq!(verdict.unwrap_err(), Refusal::PathUnresolvable);
    Ok(())
}
