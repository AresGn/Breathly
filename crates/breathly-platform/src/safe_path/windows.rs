//! Ancrage sous Windows : dossiers de la chaîne tenus ouverts sans partage de suppression (ADR 0018, note du
//! 2026-10-19). Aucun `unsafe`, aucune dépendance : `OpenOptions` de la bibliothèque standard.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use super::AnchorRefusal;

const FILE_SHARE_READ: u32 = 0x1;
const FILE_SHARE_WRITE: u32 = 0x2;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const ERROR_ACCESS_DENIED: i32 = 5;
const ERROR_SHARING_VIOLATION: i32 = 32;

/// Les handles des dossiers de la chaîne, tenus ouverts tant que l'ancre vit, et le nom de l'élément.
pub(super) struct Held {
    #[expect(
        dead_code,
        reason = "tenus ouverts : un autre processus ne peut ni renommer ni remplacer ces dossiers (ADR 0018)"
    )]
    chain: Vec<File>,
    name: OsString,
}

impl Held {
    pub(super) fn file_name(&self) -> &OsStr {
        &self.name
    }
}

pub(super) fn hold(root: &Path, below: &[&OsStr]) -> Result<Held, AnchorRefusal> {
    let Some((last, folders)) = below.split_last() else {
        return Err(AnchorRefusal::PathChanged);
    };
    let mut path = root.to_path_buf();
    let mut chain = vec![lock_folder(&path)?];
    for name in folders {
        path.push(name);
        chain.push(lock_folder(&path)?);
    }
    path.push(last);
    fs::symlink_metadata(&path).map_err(|error| refusal(&error))?;
    Ok(Held {
        chain,
        name: (*last).to_owned(),
    })
}

fn lock_folder(path: &Path) -> Result<File, AnchorRefusal> {
    let handle = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|error| refusal(&error))?;
    let attributes = handle
        .metadata()
        .map_err(|error| refusal(&error))?
        .file_attributes();
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(AnchorRefusal::PathChanged);
    }
    Ok(handle)
}

/// Violation de partage (32) : utilisé par un autre programme ; accès refusé (5) : permissions. Tout le reste : le dossier a changé ou disparu.
fn refusal(error: &io::Error) -> AnchorRefusal {
    match error.raw_os_error() {
        Some(ERROR_SHARING_VIOLATION) => AnchorRefusal::PathBusy,
        Some(ERROR_ACCESS_DENIED) => AnchorRefusal::PathDenied,
        Some(_) | None => AnchorRefusal::PathChanged,
    }
}
