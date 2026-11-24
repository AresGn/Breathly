//! Suppression et déplacement pendant que les dossiers du chemin sont tenus ouverts (ADR 0018, note Windows).
//! Un point d'analyse est retiré, jamais suivi.

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::os::windows::fs::MetadataExt;
use std::path::Path;

use super::{FILE_ATTRIBUTE_REPARSE_POINT, Held, lock_folder};
use crate::safe_path::RemovalError;

const ERROR_NOT_SAME_DEVICE: i32 = 17;

// Chaque opération reçoit `Held` : les handles de la chaîne (racine de la règle et chaque dossier) vivent donc
// pendant toute l'opération et ne peuvent pas être relâchés avant elle (ADR 0018, note Windows).

pub(super) fn byte_size(held: &Held) -> Result<u64, RemovalError> {
    measure(held.item())
}

pub(super) fn delete_permanently(held: &Held) -> Result<(), RemovalError> {
    measure(held.item())?;
    remove_tree(held.item())
}

pub(super) fn relocate(held: &Held, directory: &Path, name: &OsStr) -> Result<(), RemovalError> {
    let destination = directory.join(name);
    if fs::symlink_metadata(&destination).is_ok() {
        return Err(RemovalError::Occupied);
    }
    fs::rename(held.item(), destination).map_err(|error| map_rename(&error))
}

fn measure(path: &Path) -> Result<u64, RemovalError> {
    let meta = fs::symlink_metadata(path).map_err(|_| RemovalError::Failed)?;
    if is_reparse(&meta) || !meta.is_dir() {
        return Ok(meta.len());
    }
    let mut total = 0_u64;
    for entry in fs::read_dir(path).map_err(|_| RemovalError::Failed)? {
        let entry = entry.map_err(|_| RemovalError::Failed)?;
        total = total.saturating_add(measure(&entry.path())?);
    }
    Ok(total)
}

fn remove_tree(path: &Path) -> Result<(), RemovalError> {
    let meta = fs::symlink_metadata(path).map_err(|_| RemovalError::Failed)?;
    if is_reparse(&meta) {
        return remove_link(path, meta.is_dir());
    }
    if !meta.is_dir() {
        return fs::remove_file(path).map_err(|_| RemovalError::Failed);
    }
    let handle = lock_item(path)?;
    for entry in fs::read_dir(path).map_err(|_| RemovalError::Failed)? {
        let entry = entry.map_err(|_| RemovalError::Failed)?;
        remove_tree(&entry.path())?;
    }
    drop(handle);
    fs::remove_dir(path).map_err(|_| RemovalError::Failed)
}

fn remove_link(path: &Path, is_dir: bool) -> Result<(), RemovalError> {
    if is_dir {
        fs::remove_dir(path).map_err(|_| RemovalError::Failed)
    } else {
        fs::remove_file(path).map_err(|_| RemovalError::Failed)
    }
}

fn is_reparse(meta: &fs::Metadata) -> bool {
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

fn lock_item(path: &Path) -> Result<File, RemovalError> {
    lock_folder(path).map_err(|_| RemovalError::Failed)
}

fn map_rename(error: &io::Error) -> RemovalError {
    if error.kind() == io::ErrorKind::CrossesDevices
        || error.raw_os_error() == Some(ERROR_NOT_SAME_DEVICE)
    {
        RemovalError::CrossVolume
    } else if error.kind() == io::ErrorKind::AlreadyExists {
        RemovalError::Occupied
    } else {
        RemovalError::Failed
    }
}
