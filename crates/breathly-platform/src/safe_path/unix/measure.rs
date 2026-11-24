//! Taille d'un élément relative au descripteur du parent (ADR 0018). Les liens ne sont jamais suivis, et un autre
//! volume est refusé.

use std::ffi::{CStr, CString, OsStr, OsString};
use std::os::fd::{AsRawFd, IntoRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;

use super::{Held, open_folder_at};
use crate::safe_path::MeasureError;

pub(super) fn byte_size(held: &Held) -> Result<u64, MeasureError> {
    let info = stat_at(&held.parent, &held.name)?;
    measure(&held.parent, &held.name, info.st_dev)
}

fn measure(parent: &OwnedFd, name: &OsStr, device: u64) -> Result<u64, MeasureError> {
    let info = stat_at(parent, name)?;
    if info.st_dev != device {
        return Err(MeasureError::CrossVolume);
    }
    if is_link(info.st_mode) || !is_dir(info.st_mode) {
        return u64::try_from(info.st_size).map_err(|_| MeasureError::Failed);
    }
    let child = open_folder_at(parent, name).map_err(|_| MeasureError::Failed)?;
    let mut total = 0_u64;
    for entry in entry_names(&child)? {
        total = total.saturating_add(measure(&child, &entry, device)?);
    }
    Ok(total)
}

const MODE_MASK: u32 = 0o170_000;
const MODE_DIR: u32 = 0o040_000;
const MODE_LINK: u32 = 0o120_000;

fn is_dir(mode: libc::mode_t) -> bool {
    mode & MODE_MASK == MODE_DIR
}

fn is_link(mode: libc::mode_t) -> bool {
    mode & MODE_MASK == MODE_LINK
}

#[allow(
    unsafe_code,
    reason = "fstatat n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn stat_at(parent: &OwnedFd, name: &OsStr) -> Result<libc::stat, MeasureError> {
    let c_name = c_string(name)?;
    // SAFETY: `libc::stat` est une structure C de nombres : tout octet nul en est une valeur valide.
    let mut found: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: `parent` est un descripteur ouvert gardé par `OwnedFd` pendant l'appel, `c_name` est une chaîne C
    // vivante, et `found` est une structure valide écrite seulement par l'appel.
    let status = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            c_name.as_ptr(),
            &raw mut found,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if status < 0 {
        return Err(MeasureError::Failed);
    }
    Ok(found)
}

#[allow(
    unsafe_code,
    reason = "fdopendir n'a pas d'équivalent dans la bibliothèque standard (ADR 0018)"
)]
fn entry_names(dir: &OwnedFd) -> Result<Vec<OsString>, MeasureError> {
    let raw = dir
        .try_clone()
        .map_err(|_| MeasureError::Failed)?
        .into_raw_fd();
    // SAFETY: `raw` est un descripteur de dossier dupliqué, ouvert, que `fdopendir` prend en propriété.
    let handle = unsafe { libc::fdopendir(raw) };
    if handle.is_null() {
        // SAFETY: `fdopendir` a échoué et n'a pas pris le descripteur : il faut le fermer.
        unsafe { libc::close(raw) };
        return Err(MeasureError::Failed);
    }
    read_names(&Dir(handle))
}

#[allow(
    unsafe_code,
    reason = "readdir n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn read_names(dir: &Dir) -> Result<Vec<OsString>, MeasureError> {
    let mut names = Vec::new();
    loop {
        clear_errno();
        // SAFETY: `dir` possède un `DIR` ouvert ; `readdir` y reste valide jusqu'à `closedir` dans `Drop`.
        let entry = unsafe { libc::readdir(dir.0) };
        if entry.is_null() {
            return if errno() == 0 {
                Ok(names)
            } else {
                Err(MeasureError::Failed)
            };
        }
        // SAFETY: `readdir` a renvoyé une entrée dont `d_name` est une chaîne C terminée par un octet nul.
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
        if name.to_bytes() == b"." || name.to_bytes() == b".." {
            continue;
        }
        names.push(OsStr::from_bytes(name.to_bytes()).to_owned());
    }
}

fn c_string(name: &OsStr) -> Result<CString, MeasureError> {
    CString::new(name.as_bytes()).map_err(|_| MeasureError::Failed)
}

#[allow(
    unsafe_code,
    reason = "remettre errno à zéro avant readdir exige l'adresse du fil courant (ADR 0018)"
)]
fn clear_errno() {
    // SAFETY: `__errno_location` renvoie l'adresse de l'errno du fil courant, que l'on remet à zéro.
    unsafe { *libc::__errno_location() = 0 };
}

#[allow(
    unsafe_code,
    reason = "lire errno après readdir exige l'adresse du fil courant (ADR 0018)"
)]
fn errno() -> libc::c_int {
    // SAFETY: lecture de l'errno du fil courant, juste après `readdir`.
    unsafe { *libc::__errno_location() }
}

/// `fdopendir` a pris le descripteur : `closedir` le ferme.
struct Dir(*mut libc::DIR);

impl Drop for Dir {
    #[allow(
        unsafe_code,
        reason = "closedir ferme le DIR pris par fdopendir (ADR 0018)"
    )]
    fn drop(&mut self) {
        // SAFETY: le pointeur vient de `fdopendir` et n'est fermé qu'ici.
        unsafe { libc::closedir(self.0) };
    }
}
