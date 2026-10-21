//! Corbeille en mémoire : un faux système de fichiers (chemins et tailles) et une corbeille, sans aucun accès disque.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use crate::error::PlatformError;
use crate::trash::{TrashItem, TrashItemId, TrashProvider};
use crate::units::ByteSize;

#[derive(Default)]
struct State {
    files: BTreeMap<PathBuf, ByteSize>,
    trashed: BTreeMap<TrashItemId, TrashItem>,
    next_id: u64,
}

/// Faux `TrashProvider`. Les éléments mis à la corbeille portent la date `UNIX_EPOCH`.
#[derive(Default)]
pub struct MemoryTrash {
    state: Mutex<State>,
}

impl MemoryTrash {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un fichier au faux système de fichiers.
    pub fn add_file(&self, path: impl Into<PathBuf>, size: ByteSize) {
        self.state().files.insert(path.into(), size);
    }

    #[must_use]
    pub fn has_file(&self, path: &Path) -> bool {
        self.state().files.contains_key(path)
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Vérifie que tous les éléments sont dans la corbeille avant d'agir : tout ou rien.
fn check_all_trashed(state: &State, items: &[TrashItemId]) -> Result<(), PlatformError> {
    match items.iter().find(|id| !state.trashed.contains_key(*id)) {
        Some(missing) => Err(PlatformError::TrashItemNotFound {
            item: missing.clone(),
        }),
        None => Ok(()),
    }
}

/// Premier élément dont l'emplacement d'origine est occupé, par un fichier présent ou par un élément restauré
/// avant lui dans le même appel.
fn find_collision<'a>(state: &State, items: &'a [TrashItemId]) -> Option<&'a TrashItemId> {
    let mut seen_ids = BTreeSet::new();
    let mut claimed = BTreeSet::new();
    items.iter().filter(|id| seen_ids.insert(*id)).find(|id| {
        let path = &state.trashed[*id].original_path;
        state.files.contains_key(path) || !claimed.insert(path)
    })
}

impl TrashProvider for MemoryTrash {
    fn send(&self, path: &Path) -> Result<TrashItemId, PlatformError> {
        let mut state = self.state();
        let size = state
            .files
            .remove(path)
            .ok_or(PlatformError::TrashSourceMissing)?;
        state.next_id += 1;
        let id = TrashItemId::new(format!("memory-trash/{}", state.next_id));
        let item = TrashItem {
            id: id.clone(),
            original_path: path.to_path_buf(),
            size,
            deleted_at: SystemTime::UNIX_EPOCH,
        };
        state.trashed.insert(id.clone(), item);
        Ok(id)
    }

    fn list(&self) -> Result<Vec<TrashItem>, PlatformError> {
        Ok(self.state().trashed.values().cloned().collect())
    }

    fn restore(&self, items: &[TrashItemId]) -> Result<(), PlatformError> {
        let mut state = self.state();
        check_all_trashed(&state, items)?;
        if let Some(occupied) = find_collision(&state, items) {
            return Err(PlatformError::RestoreCollision {
                item: occupied.clone(),
            });
        }
        for id in items {
            if let Some(item) = state.trashed.remove(id) {
                state.files.insert(item.original_path, item.size);
            }
        }
        Ok(())
    }

    fn purge(&self, items: &[TrashItemId]) -> Result<(), PlatformError> {
        let mut state = self.state();
        check_all_trashed(&state, items)?;
        for id in items {
            state.trashed.remove(id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = "/fake/home/rapport.odt";
    const PHOTO: &str = "/fake/home/photo.jpg";

    fn trash_with_two_files() -> MemoryTrash {
        let trash = MemoryTrash::new();
        trash.add_file(REPORT, ByteSize::from_bytes(10));
        trash.add_file(PHOTO, ByteSize::from_bytes(20));
        trash
    }

    #[test]
    fn send_list_restore_and_purge_stay_in_memory() {
        let trash = trash_with_two_files();
        let report = trash.send(Path::new(REPORT)).unwrap();
        let photo = trash.send(Path::new(PHOTO)).unwrap();
        assert!(!trash.has_file(Path::new(REPORT)));
        let listed = trash.list().unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].original_path, Path::new(REPORT));
        assert_eq!(listed[0].size, ByteSize::from_bytes(10));

        trash.restore(std::slice::from_ref(&report)).unwrap();
        assert!(trash.has_file(Path::new(REPORT)));
        trash.purge(std::slice::from_ref(&photo)).unwrap();
        assert!(trash.list().unwrap().is_empty());
        assert!(!trash.has_file(Path::new(PHOTO)));
    }

    #[test]
    fn sending_a_missing_file_is_refused() {
        let trash = MemoryTrash::new();
        assert_eq!(
            trash.send(Path::new(REPORT)),
            Err(PlatformError::TrashSourceMissing)
        );
    }

    #[test]
    fn restore_onto_an_occupied_place_restores_nothing() {
        let trash = trash_with_two_files();
        let report = trash.send(Path::new(REPORT)).unwrap();
        let photo = trash.send(Path::new(PHOTO)).unwrap();
        trash.add_file(REPORT, ByteSize::from_bytes(99));
        let both = [photo, report.clone()];
        assert_eq!(
            trash.restore(&both),
            Err(PlatformError::RestoreCollision { item: report })
        );
        assert_eq!(trash.list().unwrap().len(), 2);
        assert!(!trash.has_file(Path::new(PHOTO)));
    }

    #[test]
    fn restoring_two_items_with_the_same_origin_restores_nothing() {
        let trash = MemoryTrash::new();
        trash.add_file(REPORT, ByteSize::from_bytes(10));
        let first = trash.send(Path::new(REPORT)).unwrap();
        trash.add_file(REPORT, ByteSize::from_bytes(11));
        let second = trash.send(Path::new(REPORT)).unwrap();
        let both = [first, second.clone()];
        assert_eq!(
            trash.restore(&both),
            Err(PlatformError::RestoreCollision { item: second })
        );
        assert_eq!(trash.list().unwrap().len(), 2);
        assert!(!trash.has_file(Path::new(REPORT)));
    }

    #[test]
    fn restoring_the_same_item_twice_in_one_call_is_not_a_collision() {
        let trash = trash_with_two_files();
        let report = trash.send(Path::new(REPORT)).unwrap();
        trash.restore(&[report.clone(), report]).unwrap();
        assert!(trash.has_file(Path::new(REPORT)));
    }

    #[test]
    fn purge_only_destroys_listed_items_and_refuses_unknown_ones() {
        let trash = trash_with_two_files();
        let report = trash.send(Path::new(REPORT)).unwrap();
        let unknown = TrashItemId::new("ailleurs".to_owned());
        assert_eq!(
            trash.purge(&[report, unknown.clone()]),
            Err(PlatformError::TrashItemNotFound { item: unknown })
        );
        assert_eq!(trash.list().unwrap().len(), 1);
    }
}
