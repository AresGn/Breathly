//! Suppression et déplacement relatifs au descripteur du parent (ADR 0018). Les liens ne sont jamais suivis.

use std::ffi::{CStr, CString, OsStr, OsString};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use libc::{AT_REMOVEDIR, AT_SYMLINK_NOFOLLOW};

use super::{DIRECTORY_FLAGS, Held, open_folder_at};
use crate::safe_path::RemovalError;

const DIRECTORY_ONLY: libc::c_int = libc::O_RDONLY | DIRECTORY_FLAGS;

pub(super) fn byte_size(held: &Held) -> Result<u64, RemovalError> {
    let info = stat_at(&held.parent, &held.name)?;
    measure(&held.parent, &held.name, info.st_dev)
}

pub(super) fn delete_permanently(held: &Held) -> Result<(), RemovalError> {
    let info = stat_at(&held.parent, &held.name)?;
    measure(&held.parent, &held.name, info.st_dev)?;
    remove_tree(&held.parent, &held.name, info.st_dev)
}

pub(super) fn relocate(held: &Held, directory: &Path, name: &OsStr) -> Result<(), RemovalError> {
    let destination = open_directory(directory)?;
    ensure_absent(&destination, name)?;
    let source = stat_at(&held.parent, &held.name)?;
    let target = device_of(&destination)?;
    if source.st_dev != target {
        return Err(RemovalError::CrossVolume);
    }
    rename_at(&Rename {
        source: &held.parent,
        source_name: &held.name,
        destination: &destination,
        destination_name: name,
    })
}

struct Rename<'a> {
    source: &'a OwnedFd,
    source_name: &'a OsStr,
    destination: &'a OwnedFd,
    destination_name: &'a OsStr,
}

fn measure(parent: &OwnedFd, name: &OsStr, device: u64) -> Result<u64, RemovalError> {
    let info = stat_at(parent, name)?;
    if info.st_dev != device {
        return Err(RemovalError::CrossVolume);
    }
    if is_link(info.st_mode) || !is_dir(info.st_mode) {
        return file_bytes(&info);
    }
    let child = open_folder_at(parent, name).map_err(|_| RemovalError::Failed)?;
    let mut total = 0_u64;
    for entry in entry_names(&child)? {
        total = total.saturating_add(measure(&child, &entry, device)?);
    }
    Ok(total)
}

fn remove_tree(parent: &OwnedFd, name: &OsStr, device: u64) -> Result<(), RemovalError> {
    let info = stat_at(parent, name)?;
    if info.st_dev != device {
        return Err(RemovalError::CrossVolume);
    }
    if is_link(info.st_mode) || !is_dir(info.st_mode) {
        return unlink_at(parent, name, 0);
    }
    let child = open_folder_at(parent, name).map_err(|_| RemovalError::Failed)?;
    for entry in entry_names(&child)? {
        remove_tree(&child, &entry, device)?;
    }
    unlink_at(parent, name, AT_REMOVEDIR)
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

fn file_bytes(info: &libc::stat) -> Result<u64, RemovalError> {
    u64::try_from(info.st_size).map_err(|_| RemovalError::Failed)
}

#[allow(
    unsafe_code,
    reason = "fstatat n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn stat_at(parent: &OwnedFd, name: &OsStr) -> Result<libc::stat, RemovalError> {
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
            AT_SYMLINK_NOFOLLOW,
        )
    };
    if status < 0 {
        return Err(RemovalError::Failed);
    }
    Ok(found)
}

#[allow(
    unsafe_code,
    reason = "unlinkat n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn unlink_at(parent: &OwnedFd, name: &OsStr, flags: libc::c_int) -> Result<(), RemovalError> {
    let c_name = c_string(name)?;
    // SAFETY: mêmes garanties que pour `fstatat` : descripteur ouvert et chaîne C terminée par un octet nul.
    let status = unsafe { libc::unlinkat(parent.as_raw_fd(), c_name.as_ptr(), flags) };
    if status < 0 {
        return Err(RemovalError::Failed);
    }
    Ok(())
}

#[allow(
    unsafe_code,
    reason = "renameat n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn rename_at(rename: &Rename<'_>) -> Result<(), RemovalError> {
    let from = c_string(rename.source_name)?;
    let to = c_string(rename.destination_name)?;
    // SAFETY: les deux descripteurs sont ouverts, les deux noms sont des chaînes C vivantes pendant l'appel.
    let status = unsafe {
        libc::renameat(
            rename.source.as_raw_fd(),
            from.as_ptr(),
            rename.destination.as_raw_fd(),
            to.as_ptr(),
        )
    };
    if status < 0 {
        return Err(rename_error());
    }
    Ok(())
}

fn rename_error() -> RemovalError {
    let code = io::Error::last_os_error().raw_os_error();
    if code == Some(libc::EXDEV) {
        RemovalError::CrossVolume
    } else {
        RemovalError::Failed
    }
}

#[allow(
    unsafe_code,
    reason = "fstatat n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn ensure_absent(parent: &OwnedFd, name: &OsStr) -> Result<(), RemovalError> {
    let c_name = c_string(name)?;
    // SAFETY: même contrat que `stat_at`.
    let mut found: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: descripteur ouvert, chaîne C vivante, structure de sortie valide.
    let status = unsafe {
        libc::fstatat(
            parent.as_raw_fd(),
            c_name.as_ptr(),
            &raw mut found,
            AT_SYMLINK_NOFOLLOW,
        )
    };
    if status == 0 {
        return Err(RemovalError::Occupied);
    }
    let error = io::Error::last_os_error();
    if error.kind() == io::ErrorKind::NotFound {
        Ok(())
    } else {
        Err(RemovalError::Failed)
    }
}

#[allow(
    unsafe_code,
    reason = "fstat n'est pas exposé sur un descripteur par la bibliothèque standard (ADR 0018)"
)]
fn device_of(dir: &OwnedFd) -> Result<u64, RemovalError> {
    // SAFETY: `libc::stat` accepte d'être mise à zéro avant l'appel.
    let mut found: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: `dir` est un descripteur de dossier ouvert pendant l'appel, `found` est valide.
    let status = unsafe { libc::fstat(dir.as_raw_fd(), &raw mut found) };
    if status < 0 {
        return Err(RemovalError::Failed);
    }
    Ok(found.st_dev)
}

#[allow(
    unsafe_code,
    reason = "open avec O_NOFOLLOW n'a pas d'équivalent dans la bibliothèque standard (ADR 0018)"
)]
fn open_directory(path: &Path) -> Result<OwnedFd, RemovalError> {
    let c_path = c_string(path.as_os_str())?;
    // SAFETY: `c_path` est une chaîne C vivante. Un descripteur positif est à nous seuls.
    let raw = unsafe { libc::open(c_path.as_ptr(), DIRECTORY_ONLY) };
    if raw < 0 {
        return Err(RemovalError::Failed);
    }
    // SAFETY: `open` vient de renvoyer ce descripteur, ouvert et non partagé.
    Ok(unsafe { OwnedFd::from_raw_fd(raw) })
}

#[allow(
    unsafe_code,
    reason = "fdopendir n'a pas d'équivalent dans la bibliothèque standard (ADR 0018)"
)]
fn entry_names(dir: &OwnedFd) -> Result<Vec<OsString>, RemovalError> {
    let raw = dir
        .try_clone()
        .map_err(|_| RemovalError::Failed)?
        .into_raw_fd();
    // SAFETY: `raw` est un descripteur de dossier dupliqué, ouvert, que `fdopendir` prend en propriété.
    let handle = unsafe { libc::fdopendir(raw) };
    if handle.is_null() {
        // SAFETY: `fdopendir` a échoué et n'a pas pris le descripteur : il faut le fermer.
        unsafe { libc::close(raw) };
        return Err(RemovalError::Failed);
    }
    let dir = Dir(handle);
    read_names(&dir)
}

#[allow(
    unsafe_code,
    reason = "readdir n'a pas d'équivalent relatif à un descripteur dans la bibliothèque standard (ADR 0018)"
)]
fn read_names(dir: &Dir) -> Result<Vec<OsString>, RemovalError> {
    let mut names = Vec::new();
    loop {
        clear_errno();
        // SAFETY: `dir` possède un `DIR` ouvert ; `readdir` y reste valide jusqu'à `closedir` dans `Drop`.
        let entry = unsafe { libc::readdir(dir.0) };
        if entry.is_null() {
            return if errno() == 0 {
                Ok(names)
            } else {
                Err(RemovalError::Failed)
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

fn c_string(name: &OsStr) -> Result<CString, RemovalError> {
    CString::new(name.as_bytes()).map_err(|_| RemovalError::Failed)
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
