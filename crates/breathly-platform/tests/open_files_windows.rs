//! Sonde d'ouverture avec accès `DELETE` sous Windows (ticket #73, ADR 0020). Non vérifié à ce jour : ce fichier
//! n'est compilé que sous Windows, où il doit être joué à la main sur la machine de validation.
#![cfg(windows)]

use std::error::Error;
use std::fs::{self, OpenOptions};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use breathly_platform::{CanonicalPath, OpenFileInspector, OpenFileStatus, SystemOpenFiles};

type TestResult = Result<(), Box<dyn Error>>;

fn status(path: &Path) -> Result<OpenFileStatus, Box<dyn Error>> {
    Ok(SystemOpenFiles::new().check(&CanonicalPath::resolve(path)?))
}

#[test]
fn a_file_held_without_delete_sharing_is_in_use_then_free_once_closed() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let file = sandbox.path().join("held.bin");
    fs::write(&file, b"x")?;
    let held = OpenOptions::new().read(true).share_mode(0).open(&file)?;
    assert_eq!(
        status(&file)?,
        OpenFileStatus::InUse {
            programs: Vec::new()
        }
    );
    drop(held);
    assert_eq!(status(&file)?, OpenFileStatus::Free);
    Ok(())
}

#[test]
fn a_folder_holding_such_a_file_is_in_use() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let folder = sandbox.path().join("cache");
    fs::create_dir_all(&folder)?;
    let file = folder.join("held.bin");
    fs::write(&file, b"x")?;
    let _held = OpenOptions::new().read(true).share_mode(0).open(&file)?;
    assert!(matches!(status(&folder)?, OpenFileStatus::InUse { .. }));
    Ok(())
}
