//! Ancrage sous Linux : `openat` avec `O_NOFOLLOW` à chaque niveau (ADR 0018).

use std::ffi::{CString, OsStr, OsString};
use std::fs::OpenOptions;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

mod remove;

use super::AnchorRefusal;
use super::RemovalError;

const DIRECTORY_FLAGS: libc::c_int =
    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC;

/// Le dossier parent de l'élément, tenu ouvert, et le nom de l'élément.
pub(super) struct Held {
    parent: OwnedFd,
    name: OsString,
}

impl Held {
    pub(super) fn file_name(&self) -> &OsStr {
        &self.name
    }

    pub(super) fn byte_size(&self) -> Result<u64, RemovalError> {
        remove::byte_size(self)
    }

    pub(super) fn delete_permanently(&self) -> Result<(), RemovalError> {
        remove::delete_permanently(self)
    }

    pub(super) fn relocate(&self, directory: &Path, name: &OsStr) -> Result<(), RemovalError> {
        remove::relocate(self, directory, name)
    }
}

pub(super) fn hold(root: &Path, below: &[&OsStr]) -> Result<Held, AnchorRefusal> {
    let Some((last, folders)) = below.split_last() else {
        return Err(AnchorRefusal::PathChanged);
    };
    let mut dir = open_root(root)?;
    for name in folders {
        dir = open_folder_at(&dir, name)?;
    }
    check_entry_at(&dir, last)?;
    Ok(Held {
        parent: dir,
        name: (*last).to_owned(),
    })
}

fn open_root(root: &Path) -> Result<OwnedFd, AnchorRefusal> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
        .open(root)
        .map(OwnedFd::from)
        .map_err(|error| refusal(&error))
}

#[allow(
    unsafe_code,
    reason = "openat n'a pas d'équivalent dans la bibliothèque standard (ADR 0018)"
)]
fn open_folder_at(parent: &OwnedFd, name: &OsStr) -> Result<OwnedFd, AnchorRefusal> {
    let c_name = CString::new(name.as_bytes()).map_err(|_| AnchorRefusal::PathChanged)?;
    // SAFETY: `parent` est un descripteur ouvert que `OwnedFd` garde valide pendant l'appel, et `c_name` est une
    // chaîne C terminée par un octet nul, vivante jusqu'à la fin de l'appel.
    let raw = unsafe { libc::openat(parent.as_raw_fd(), c_name.as_ptr(), DIRECTORY_FLAGS) };
    if raw < 0 {
        return Err(refusal(&io::Error::last_os_error()));
    }
    // SAFETY: `openat` vient de renvoyer ce descripteur, ouvert et à nous seuls : `OwnedFd` en prend la propriété.
    Ok(unsafe { OwnedFd::from_raw_fd(raw) })
}

#[allow(
    unsafe_code,
    reason = "fstatat n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn check_entry_at(parent: &OwnedFd, name: &OsStr) -> Result<(), AnchorRefusal> {
    let c_name = CString::new(name.as_bytes()).map_err(|_| AnchorRefusal::PathChanged)?;
    // SAFETY: `libc::stat` est une structure C de nombres entiers : tout octet nul en est une valeur valide.
    let mut found: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: mêmes garanties que pour `openat` ; `found` est une structure valide, écrite par l'appel seulement.
    let status = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            c_name.as_ptr(),
            &raw mut found,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if status < 0 {
        return Err(refusal(&io::Error::last_os_error()));
    }
    Ok(())
}

/// Un refus de permission est « inaccessible », `EBUSY` « occupé » ; un lien (`ELOOP`), un non-dossier ou une disparition, « changé ».
fn refusal(error: &io::Error) -> AnchorRefusal {
    match error.raw_os_error() {
        Some(code) if code == libc::EACCES || code == libc::EPERM => AnchorRefusal::PathDenied,
        Some(code) if code == libc::EBUSY => AnchorRefusal::PathBusy,
        Some(_) | None => AnchorRefusal::PathChanged,
    }
}
