//! Banc d'essai commun des tests des garde-fous : un dossier temporaire et les faux de `breathly-platform`.
#![allow(
    dead_code,
    reason = "chaque fichier de tests n'utilise qu'une partie du banc"
)]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use breathly_disk::{Candidate, Guardrails, Refusal};
use breathly_platform::fake::{FakeOpenFiles, FixedStandardPaths};
use breathly_rules::{Rule, parse_cleanup_rules};
use tempfile::TempDir;

pub type TestResult = Result<(), Box<dyn Error>>;

pub struct Fixture {
    pub sandbox: TempDir,
    pub paths: FixedStandardPaths,
    pub open_files: FakeOpenFiles,
}

impl Fixture {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let sandbox = tempfile::tempdir()?;
        fs::create_dir_all(sandbox.path().join("home"))?;
        let paths = FixedStandardPaths::under(sandbox.path());
        Ok(Self {
            sandbox,
            paths,
            open_files: FakeOpenFiles::new(),
        })
    }

    pub fn home(&self) -> PathBuf {
        self.sandbox.path().join("home")
    }

    pub fn file(&self, relative: &str) -> Result<PathBuf, Box<dyn Error>> {
        let path = self.home().join(relative);
        fs::create_dir_all(path.parent().ok_or("chemin sans parent")?)?;
        fs::write(&path, b"contenu")?;
        Ok(path)
    }

    pub fn verdict(&self, rule: &Rule, place: (&[PathBuf], &Path)) -> Result<(), Refusal> {
        let (roots, path) = place;
        verdict_of(self, Candidate { rule, roots, path })
    }
}

pub fn verdict_of(fixture: &Fixture, candidate: Candidate<'_>) -> Result<(), Refusal> {
    Guardrails::new(&fixture.paths, &fixture.open_files)
        .verify(&candidate)
        .map(drop)
}

pub fn rule(risk: &str) -> Result<Rule, Box<dyn Error>> {
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

pub fn refused_in_home(fixture: &Fixture, relative: &str) -> Result<Refusal, Box<dyn Error>> {
    let item = fixture.file(relative)?;
    let roots = [fixture.home()];
    fixture
        .verdict(&rule("safe")?, (&roots, &item))
        .err()
        .ok_or_else(|| format!("{relative} aurait dû être refusé").into())
}

/// Des dossiers standard dont `Videos` est inconnu de l'OS : un garde-fou qui ne peut pas contrôler refuse.
pub struct PathsWithoutVideos<'a>(pub &'a FixedStandardPaths);

impl breathly_platform::StandardPaths for PathsWithoutVideos<'_> {
    fn config_dir(&self) -> Result<PathBuf, breathly_platform::PlatformError> {
        self.0.config_dir()
    }
    fn state_dir(&self) -> Result<PathBuf, breathly_platform::PlatformError> {
        self.0.state_dir()
    }
    fn home_dir(&self) -> Result<PathBuf, breathly_platform::PlatformError> {
        self.0.home_dir()
    }
    fn temp_dir(&self) -> Result<PathBuf, breathly_platform::PlatformError> {
        self.0.temp_dir()
    }
    fn user_folder(
        &self,
        folder: breathly_platform::UserFolder,
    ) -> Result<PathBuf, breathly_platform::PlatformError> {
        match folder {
            breathly_platform::UserFolder::Videos => {
                Err(breathly_platform::PlatformError::StandardPathUnavailable {
                    path: breathly_platform::StandardPath::Folder(folder),
                })
            }
            breathly_platform::UserFolder::Documents
            | breathly_platform::UserFolder::Pictures
            | breathly_platform::UserFolder::Music
            | breathly_platform::UserFolder::Desktop => self.0.user_folder(folder),
        }
    }
}
