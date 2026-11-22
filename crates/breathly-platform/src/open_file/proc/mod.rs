//! Fichier ouvert sous Linux : instantané de `/proc` (ADR 0020, essai du 2026-10-19). Pour **chaque** vérification,
//! les descripteurs (`/proc/<pid>/fd`) et les cartes mémoire (`/proc/<pid>/maps`) de chaque processus lisible, hors
//! Breathly lui-même. Un processus illisible (autre compte, processus protégé) est ignoré : la limite est connue.
//! Seul le processus de Breathly lui-même est exclu, pas ses descendants : un processus enfant qui tient le fichier
//! est présenté comme un logiciel à fermer, ce qui refuse par prudence (jamais l'inverse).
//! Si `/proc` est inutilisable (absent, ou pas même les descripteurs de Breathly lisibles), tout est « impossible ».
//! Aucun `unsafe`, aucun programme externe.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::{OpenFileInspector, OpenFileStatus};
use crate::safe_path::CanonicalPath;
use crate::units::{ProcessId, ProcessName};

const PROC: &str = "/proc";
const DELETED_SUFFIX: &str = " (deleted)";

pub(super) fn check(item: &CanonicalPath) -> OpenFileStatus {
    ProcFiles::system().check(item)
}

/// Lecture d'un arbre `/proc`. Le constructeur d'un autre arbre sert aux tests (`proc` fabriqué en dossier
/// temporaire) ; le seul adapter public est `SystemOpenFiles`.
pub(super) struct ProcFiles {
    root: PathBuf,
    own: ProcessId,
}

/// Un fichier tenu par un processus : son identité et son chemin tels que le noyau les donne.
struct Held {
    identity: (u64, u64),
    path: PathBuf,
    process: ProcessId,
}

impl ProcFiles {
    fn system() -> Self {
        Self::under(PathBuf::from(PROC), ProcessId::new(std::process::id()))
    }

    pub(super) const fn under(root: PathBuf, own: ProcessId) -> Self {
        Self { root, own }
    }

    /// Tous les fichiers tenus par les processus lisibles, hors Breathly ; `None` si `/proc` est inutilisable.
    fn snapshot(&self) -> Option<Vec<Held>> {
        let own_fds = self.root.join(self.own.get().to_string()).join("fd");
        fs::read_dir(own_fds).ok()?;
        let mut held = Vec::new();
        for pid in self.processes()? {
            if pid != self.own {
                held.extend(self.files_of(pid));
            }
        }
        Some(held)
    }

    fn processes(&self) -> Option<Vec<ProcessId>> {
        let entries = fs::read_dir(&self.root).ok()?;
        Some(
            entries
                .flatten()
                .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
                .map(ProcessId::new)
                .collect(),
        )
    }

    fn files_of(&self, pid: ProcessId) -> Vec<Held> {
        let base = self.root.join(pid.get().to_string());
        let mut held = descriptors(&base, pid);
        held.extend(mappings(&base, pid));
        held
    }

    fn program_name(&self, pid: ProcessId) -> Option<ProcessName> {
        let comm = fs::read_to_string(self.root.join(pid.get().to_string()).join("comm")).ok()?;
        ProcessName::try_from(comm.trim().to_owned()).ok()
    }
}

impl OpenFileInspector for ProcFiles {
    fn check(&self, item: &CanonicalPath) -> OpenFileStatus {
        let Some(snapshot) = self.snapshot() else {
            return OpenFileStatus::Unavailable;
        };
        let Ok(metadata) = fs::metadata(item.as_path()) else {
            return OpenFileStatus::Unavailable;
        };
        let target = Target {
            identity: (metadata.dev(), metadata.ino()),
            path: item.as_path(),
            is_folder: metadata.is_dir(),
        };
        let holders: BTreeSet<ProcessId> = snapshot
            .iter()
            .filter(|held| held.covers(&target))
            .map(|held| held.process)
            .collect();
        if holders.is_empty() {
            return OpenFileStatus::Free;
        }
        let programs: BTreeSet<ProcessName> = holders
            .into_iter()
            .filter_map(|pid| self.program_name(pid))
            .collect();
        OpenFileStatus::InUse {
            programs: programs.into_iter().collect(),
        }
    }
}

/// L'élément vérifié.
struct Target<'a> {
    identity: (u64, u64),
    path: &'a Path,
    is_folder: bool,
}

impl Held {
    /// Un fichier est tenu si son identité (dev, inode) ou son chemin figure dans l'instantané ; un dossier, si un
    /// chemin de l'instantané se trouve dessous. Le chemin est un second critère : le périphérique de `maps` est
    /// celui du superbloc, qui peut différer de celui de `stat` (sous-volume btrfs, overlayfs) ; une
    /// correspondance de trop refuse un élément, jamais n'en laisse passer un.
    fn covers(&self, target: &Target<'_>) -> bool {
        self.identity == target.identity
            || self.path == target.path
            || (target.is_folder && self.path.starts_with(target.path))
    }
}

/// Les descripteurs d'un processus qui désignent un fichier (pas un tube ni une prise : leur cible n'est pas un
/// chemin absolu). La lecture suit le lien magique : elle donne l'identité même d'un fichier supprimé.
fn descriptors(base: &Path, pid: ProcessId) -> Vec<Held> {
    let Ok(entries) = fs::read_dir(base.join("fd")) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = clean(&fs::read_link(entry.path()).ok()?)?;
            let metadata = fs::metadata(entry.path()).ok()?;
            Some(Held {
                identity: (metadata.dev(), metadata.ino()),
                path,
                process: pid,
            })
        })
        .collect()
}

/// Les fichiers cartographiés en mémoire, dont le descripteur est peut-être fermé (`/proc/<pid>/maps`). Lu en
/// octets : un nom de fichier n'est pas toujours de l'UTF-8, et il ne doit pas faire disparaître l'entrée.
fn mappings(base: &Path, pid: ProcessId) -> Vec<Held> {
    let Ok(bytes) = fs::read(base.join("maps")) else {
        return Vec::new();
    };
    bytes
        .split(|byte| *byte == b'\n')
        .filter_map(|line| mapping(line, pid))
        .collect()
}

/// Une ligne : `adresses droits décalage major:minor inode  chemin` ; le chemin peut contenir des espaces.
fn mapping(line: &[u8], pid: ProcessId) -> Option<Held> {
    let mut fields = line.splitn(6, |byte| *byte == b' ');
    let device = std::str::from_utf8(fields.nth(3)?).ok()?;
    let inode: u64 = std::str::from_utf8(fields.next()?)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let path = clean(Path::new(OsStr::from_bytes(
        fields.next()?.trim_ascii_start(),
    )))?;
    let (major, minor) = device.split_once(':')?;
    let device = libc::makedev(
        u32::from_str_radix(major, 16).ok()?,
        u32::from_str_radix(minor, 16).ok()?,
    );
    Some(Held {
        identity: (device, inode),
        path,
        process: pid,
    })
}

/// Un chemin absolu, sans le suffixe « (deleted) » que le noyau ajoute à un fichier supprimé. Travaille en octets :
/// un nom qui n'est pas de l'UTF-8 reste reconnu.
fn clean(path: &Path) -> Option<PathBuf> {
    let bytes = path.as_os_str().as_bytes();
    if bytes.first() != Some(&b'/') {
        return None;
    }
    let bytes = bytes
        .strip_suffix(DELETED_SUFFIX.as_bytes())
        .unwrap_or(bytes);
    Some(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(test)]
mod tests;
