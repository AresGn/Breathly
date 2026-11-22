//! Les garde-fous S5 qui dépendent du disque, revérifiés pour chaque élément (SEC2). Aucun ne se désactive : aucune
//! fonction de ce module ne lit un réglage.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use breathly_platform::{CanonicalPath, StandardPaths, UserFolder};
use breathly_rules::is_protected_folder;

use super::browsers::PROFILE_LOCATIONS;
use super::refusal::Refusal;

const ENCRYPTED_EXTENSIONS: [&str; 3] = ["hc", "vc", "kdbx"];
const USER_FOLDERS: [UserFolder; 5] = [
    UserFolder::Documents,
    UserFolder::Pictures,
    UserFolder::Videos,
    UserFolder::Music,
    UserFolder::Desktop,
];

/// Contrôle `item` (chemin canonique) contre tous les emplacements et noms protégés.
pub(super) fn check(item: &CanonicalPath, paths: &dyn StandardPaths) -> Result<(), Refusal> {
    let home = CanonicalPath::resolve(&paths.home_dir()?)?;
    check_user_folders(item, &home, paths)?;
    check_browser_profiles(item, &home)?;
    check_git_and_containers(item)
}

fn check_user_folders(
    item: &CanonicalPath,
    home: &CanonicalPath,
    paths: &dyn StandardPaths,
) -> Result<(), Refusal> {
    let known = USER_FOLDERS
        .into_iter()
        .filter_map(|folder| paths.user_folder(folder).ok());
    let under_home = protected_names(home);
    let overlaps = known
        .chain(under_home)
        .filter_map(|folder| CanonicalPath::resolve(&folder).ok())
        .any(|folder| overlap(item, &folder));
    if overlaps {
        Err(Refusal::UserFolder)
    } else {
        Ok(())
    }
}

/// Les dossiers protégés de la maison, par leur nom français ou anglais (même s'ils n'ont pas de dossier standard).
fn protected_names(home: &CanonicalPath) -> Vec<PathBuf> {
    fs::read_dir(home.as_path())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| is_protected_folder(&entry.file_name().to_string_lossy()))
        .map(|entry| entry.path())
        .collect()
}

fn check_browser_profiles(item: &CanonicalPath, home: &CanonicalPath) -> Result<(), Refusal> {
    let overlaps = PROFILE_LOCATIONS
        .iter()
        .filter_map(|location| CanonicalPath::resolve(&home.as_path().join(location)).ok())
        .any(|profile| overlap(item, &profile));
    if overlaps {
        Err(Refusal::BrowserProfile)
    } else {
        Ok(())
    }
}

/// Vrai si l'un contient l'autre : supprimer un dossier parent détruirait aussi le dossier protégé.
fn overlap(item: &CanonicalPath, protected: &CanonicalPath) -> bool {
    item.is_within(protected) || protected.is_within(item)
}

fn check_git_and_containers(item: &CanonicalPath) -> Result<(), Refusal> {
    let path = item.as_path();
    if path.components().any(|part| is_git(part.as_os_str())) {
        return Err(Refusal::GitFolder);
    }
    if is_encrypted_container(path) {
        return Err(Refusal::EncryptedContainer);
    }
    if path.is_dir() {
        return check_contents(path);
    }
    Ok(())
}

/// Parcourt un dossier sans suivre de lien : un `.git` ou un conteneur chiffré dedans protège tout le dossier.
fn check_contents(folder: &Path) -> Result<(), Refusal> {
    let mut pending = vec![folder.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries = fs::read_dir(&current).map_err(|_| Refusal::PathBusy)?;
        for entry in entries {
            let entry = entry.map_err(|_| Refusal::PathBusy)?;
            check_entry(&entry, &mut pending)?;
        }
    }
    Ok(())
}

fn check_entry(entry: &fs::DirEntry, pending: &mut Vec<PathBuf>) -> Result<(), Refusal> {
    if is_git(&entry.file_name()) {
        return Err(Refusal::GitFolder);
    }
    if is_encrypted_container(&entry.path()) {
        return Err(Refusal::EncryptedContainer);
    }
    if entry.file_type().map_err(|_| Refusal::PathBusy)?.is_dir() {
        pending.push(entry.path());
    }
    Ok(())
}

fn is_git(name: &OsStr) -> bool {
    name.to_string_lossy().eq_ignore_ascii_case(".git")
}

fn is_encrypted_container(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        ENCRYPTED_EXTENSIONS
            .iter()
            .any(|known| extension.to_string_lossy().eq_ignore_ascii_case(known))
    })
}
