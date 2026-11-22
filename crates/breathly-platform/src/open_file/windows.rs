//! Fichier ouvert sous Windows : une ouverture avec accès `DELETE` et partage total échoue (code 32) exactement
//! quand la suppression échouerait (ADR 0020, essai du 2026-10-19). Un logiciel qui a ouvert le fichier avec
//! partage de suppression n'est **pas** vu : Windows autorise alors sa suppression. Windows ne dit pas qui tient
//! le fichier : la liste des logiciels est vide. Aucun `unsafe` : `OpenOptions` de la bibliothèque standard.
//!
//! Essayé à la main sur Windows 11 seulement ; ce module n'est pas exécuté par la CI Linux.

use std::fs::{self, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use super::OpenFileStatus;
use crate::safe_path::CanonicalPath;

const DELETE: u32 = 0x0001_0000;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_SHARE_ALL: u32 = 0x1 | 0x2 | 0x4;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const ERROR_SHARING_VIOLATION: i32 = 32;

pub(super) fn check(item: &CanonicalPath) -> OpenFileStatus {
    let Ok(metadata) = fs::symlink_metadata(item.as_path()) else {
        return OpenFileStatus::Unavailable;
    };
    let outcome = if metadata.is_dir() {
        scan_folder(item.as_path())
    } else {
        probe(item.as_path())
    };
    outcome.map_or_else(|status| status, |()| OpenFileStatus::Free)
}

/// Un dossier est utilisé si l'un des fichiers dessous l'est ; un dossier illisible est « impossible ».
fn scan_folder(folder: &Path) -> Result<(), OpenFileStatus> {
    let mut pending = vec![folder.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries = fs::read_dir(&current).map_err(|_| OpenFileStatus::Unavailable)?;
        for entry in entries {
            let entry = entry.map_err(|_| OpenFileStatus::Unavailable)?;
            match classify(&entry) {
                Entry::File(path) => probe(&path)?,
                Entry::Folder(path) => pending.push(path),
                Entry::Link => {}
            }
        }
    }
    Ok(())
}

enum Entry {
    File(PathBuf),
    Folder(PathBuf),
    /// Lien ou jonction : jamais suivi (ADR 0018), donc jamais sondé.
    Link,
}

fn classify(entry: &fs::DirEntry) -> Entry {
    match entry.file_type() {
        Ok(kind) if kind.is_symlink() => Entry::Link,
        Ok(kind) if kind.is_dir() => Entry::Folder(entry.path()),
        Ok(_) | Err(_) => Entry::File(entry.path()),
    }
}

fn probe(path: &Path) -> Result<(), OpenFileStatus> {
    OpenOptions::new()
        .access_mode(DELETE | FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_ALL)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map(drop)
        .map_err(|error| failure(&error))
}

fn failure(error: &io::Error) -> OpenFileStatus {
    if error.raw_os_error() == Some(ERROR_SHARING_VIOLATION) {
        OpenFileStatus::InUse {
            programs: Vec::new(),
        }
    } else {
        OpenFileStatus::Unavailable
    }
}
