//! Les garde-fous S5 qui dépendent du disque, revérifiés pour chaque élément (SEC2). Aucun ne se désactive : aucune
//! fonction de ce module ne lit un réglage.

use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use breathly_platform::{CanonicalPath, StandardPaths, UserFolder};
use breathly_rules::{is_encrypted_container_extension, is_protected_folder};

use super::browsers::{
    FIREFOX_PACKAGE_PREFIX, FIREFOX_PACKAGE_PROFILE, PACKAGES_FOLDER, PROFILE_LOCATIONS,
};
use super::refusal::Refusal;

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
    let mut folders = Vec::new();
    for folder in USER_FOLDERS {
        folders.push(paths.user_folder(folder)?);
    }
    folders.extend(protected_names(home)?);
    overlaps_any(item, &folders, Refusal::UserFolder)
}

/// Les dossiers protégés de la maison, par leur nom français ou anglais (même s'ils n'ont pas de dossier standard).
fn protected_names(home: &CanonicalPath) -> Result<Vec<PathBuf>, Refusal> {
    let mut found = Vec::new();
    for entry in fs::read_dir(home.as_path()).map_err(|_| Refusal::PathBusy)? {
        let entry = entry.map_err(|_| Refusal::PathBusy)?;
        if is_protected_folder(&entry.file_name().to_string_lossy()) {
            found.push(entry.path());
        }
    }
    Ok(found)
}

fn check_browser_profiles(item: &CanonicalPath, home: &CanonicalPath) -> Result<(), Refusal> {
    let base = home.as_path();
    let mut profiles: Vec<PathBuf> = PROFILE_LOCATIONS
        .iter()
        .map(|location| base.join(location))
        .collect();
    profiles.extend(store_firefox_profiles(&base.join(PACKAGES_FOLDER))?);
    overlaps_any(item, &profiles, Refusal::BrowserProfile)
}

/// Firefox du Microsoft Store : le dossier du paquet porte un identifiant d'éditeur, d'où la comparaison par préfixe.
fn store_firefox_profiles(packages: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let Some(entries) = read_dir_if_present(packages)? else {
        return Ok(Vec::new());
    };
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| Refusal::PathBusy)?;
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with(FIREFOX_PACKAGE_PREFIX) {
            found.push(entry.path().join(FIREFOX_PACKAGE_PROFILE));
        }
    }
    Ok(found)
}

fn read_dir_if_present(folder: &Path) -> Result<Option<fs::ReadDir>, Refusal> {
    match fs::read_dir(folder) {
        Ok(entries) => Ok(Some(entries)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(_) => Err(Refusal::PathBusy),
    }
}

/// Refuse si `item` recoupe l'un des dossiers protégés. Un dossier absent n'a rien à protéger ; un dossier présent
/// mais impossible à résoudre fait refuser l'élément (échec fermé).
fn overlaps_any(
    item: &CanonicalPath,
    folders: &[PathBuf],
    refusal: Refusal,
) -> Result<(), Refusal> {
    for folder in folders {
        match CanonicalPath::resolve(folder) {
            Ok(resolved) if overlap(item, &resolved) => return Err(refusal),
            Ok(_) => {}
            Err(_) if matches!(folder.try_exists(), Ok(false)) => {}
            Err(_) => return Err(Refusal::PathUnresolvable),
        }
    }
    Ok(())
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
    path.extension()
        .is_some_and(|extension| is_encrypted_container_extension(&extension.to_string_lossy()))
}
