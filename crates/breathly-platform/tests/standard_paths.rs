//! L'adapter réel sous Linux, variables d'environnement redirigées vers un dossier temporaire (ticket #33).
//!
//! Le test relance son propre binaire avec un environnement vidé puis redirigé : le processus des tests garde le
//! sien, et ni le vrai dossier personnel ni les vrais réglages ne sont lus.
#![cfg(target_os = "linux")]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use breathly_platform::{StandardPaths, SystemStandardPaths, UserFolder};

type TestResult = Result<(), Box<dyn Error>>;

const SANDBOX: &str = "BREATHLY_TEST_SANDBOX";
const CHILD: &str = "the_real_adapter_reads_the_redirected_environment";

#[test]
fn the_real_adapter_reads_the_redirected_environment() -> TestResult {
    let Some(root) = std::env::var_os(SANDBOX).map(PathBuf::from) else {
        return Ok(());
    };
    let paths = SystemStandardPaths::new();
    assert_eq!(paths.home_dir()?, root.join("home"));
    assert_eq!(paths.config_dir()?, root.join("config/breathly"));
    assert_eq!(paths.state_dir()?, root.join("state/breathly"));
    assert_eq!(paths.temp_dir()?, root.join("tmp"));
    assert_eq!(
        paths.user_folder(UserFolder::Documents)?,
        root.join("home/Documents")
    );
    assert_eq!(
        paths.user_folder(UserFolder::Desktop)?,
        root.join("home/Bureau")
    );
    Ok(())
}

#[test]
fn the_redirected_environment_is_checked_in_a_child_process() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path();
    prepare(root)?;
    #[allow(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "relance du binaire de test lui-même, pour isoler l'environnement redirigé"
    )]
    let status = std::process::Command::new(std::env::current_exe()?)
        .args(["--exact", CHILD, "--quiet"])
        .env_clear()
        .env(SANDBOX, root)
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("TMPDIR", root.join("tmp"))
        .status()?;
    assert!(status.success(), "le processus enfant a échoué : {status}");
    Ok(())
}

fn prepare(root: &Path) -> TestResult {
    for dir in ["home/Documents", "home/Bureau", "config", "state", "tmp"] {
        fs::create_dir_all(root.join(dir))?;
    }
    fs::write(
        root.join("config/user-dirs.dirs"),
        "XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\nXDG_DESKTOP_DIR=\"$HOME/Bureau\"\n",
    )?;
    Ok(())
}
