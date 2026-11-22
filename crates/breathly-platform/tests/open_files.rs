//! Fichier réellement ouvert par un autre processus, dans un dossier temporaire (ticket #73, ADR 0020).
#![cfg(target_os = "linux")]
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "le programme d'essai est un processus enfant qui tient un fichier ouvert : seul ce crate peut en lancer un (SEC6)"
)]

use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use breathly_platform::{CanonicalPath, OpenFileInspector, OpenFileStatus, SystemOpenFiles};

type TestResult = Result<(), Box<dyn Error>>;

/// Un processus enfant qui ouvre `path` en lecture-écriture puis attend ; tué à la fin du test.
struct Holder(Child);

impl Holder {
    fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut child = Command::new("sh")
            .args(["-c", r#"exec 3<>"$1"; echo ready; exec sleep 30"#, "sh"])
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().ok_or("sortie de l'enfant absente")?;
        let mut line = String::new();
        BufReader::new(stdout).read_line(&mut line)?;
        Ok(Self(child))
    }

    fn close(mut self) -> TestResult {
        self.0.kill()?;
        self.0.wait()?;
        Ok(())
    }
}

impl Drop for Holder {
    fn drop(&mut self) {
        drop(self.0.kill());
    }
}

fn status(path: &Path) -> Result<OpenFileStatus, Box<dyn Error>> {
    Ok(SystemOpenFiles::new().check(&CanonicalPath::resolve(path)?))
}

#[test]
fn a_file_really_open_in_another_process_is_in_use_then_free_once_closed() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let file = sandbox.path().join("held.bin");
    fs::write(&file, b"x")?;
    let free = sandbox.path().join("free.bin");
    fs::write(&free, b"x")?;
    let holder = Holder::open(&file)?;
    assert!(matches!(status(&file)?, OpenFileStatus::InUse { programs } if !programs.is_empty()));
    assert_eq!(status(&free)?, OpenFileStatus::Free);
    holder.close()?;
    assert_eq!(status(&file)?, OpenFileStatus::Free);
    Ok(())
}

#[test]
fn a_folder_holding_a_really_open_file_is_in_use() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let folder = sandbox.path().join("cache");
    fs::create_dir_all(&folder)?;
    let file = folder.join("held.bin");
    fs::write(&file, b"x")?;
    let _holder = Holder::open(&file)?;
    assert!(matches!(status(&folder)?, OpenFileStatus::InUse { .. }));
    Ok(())
}
