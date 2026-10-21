//! Port `TrashProvider` : la corbeille du système (ADR 0005, ADR 0018).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::PlatformError;
use crate::units::ByteSize;

/// Identifiant opaque d'un élément de la corbeille (sous Linux, le chemin de son fichier `.trashinfo`, ADR 0005).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrashItemId(String);

impl TrashItemId {
    #[must_use]
    pub const fn new(raw: String) -> Self {
        Self(raw)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Élément présent dans la corbeille, mis par Breathly ou non.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashItem {
    pub id: TrashItemId,
    pub original_path: PathBuf,
    pub size: ByteSize,
    pub deleted_at: SystemTime,
}

/// La corbeille du système.
///
/// `send` reçoit un chemin : seul `disk-clean` l'appelle, avec le chemin de la quarantaine atteint par le parcours
/// sans suivre de lien (ADR 0018). L'interface n'envoie jamais de chemin (SEC3).
pub trait TrashProvider: Send + Sync {
    /// Envoie l'élément à la corbeille.
    ///
    /// # Errors
    /// `TrashSourceMissing` si l'élément n'existe pas, `TrashUnavailable` si la corbeille refuse.
    fn send(&self, path: &Path) -> Result<TrashItemId, PlatformError>;

    /// Liste tous les éléments de la corbeille.
    ///
    /// # Errors
    /// `TrashUnavailable` si la corbeille ne peut pas être lue.
    fn list(&self) -> Result<Vec<TrashItem>, PlatformError>;

    /// Remet les éléments à leur emplacement d'origine ; n'écrase jamais rien (US-041.2).
    ///
    /// # Errors
    /// `TrashItemNotFound`, `RestoreCollision` (rien n'est restauré) ou `TrashUnavailable`.
    fn restore(&self, items: &[TrashItemId]) -> Result<(), PlatformError>;

    /// Détruit définitivement les éléments listés, et eux seuls (ADR 0005).
    ///
    /// # Errors
    /// `TrashItemNotFound` (rien n'est détruit) ou `TrashUnavailable`.
    fn purge(&self, items: &[TrashItemId]) -> Result<(), PlatformError>;
}
