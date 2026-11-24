//! Échec d'une suppression ou d'un déplacement relatif à une ancre (ADR 0018). Aucune variante ne porte de chemin.

use thiserror::Error;

/// La suppression n'a pas eu lieu, ou elle s'est arrêtée sans traverser un lien (échec fermé).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RemovalError {
    #[error("l'élément n'est pas sur le même volume que la quarantaine")]
    CrossVolume,
    #[error("l'emplacement de destination est déjà occupé")]
    Occupied,
    #[error("la suppression a échoué")]
    Failed,
}

impl RemovalError {
    /// Code stable, repris dans l'historique et les journaux.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.names().0
    }

    /// Clé du texte affiché, dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(self) -> &'static str {
        self.names().1
    }

    const fn names(self) -> (&'static str, &'static str) {
        match self {
            Self::CrossVolume => (
                "platform.removal_cross_volume",
                "errors.platform.removal_cross_volume",
            ),
            Self::Occupied => (
                "platform.removal_occupied",
                "errors.platform.removal_occupied",
            ),
            Self::Failed => ("platform.removal_failed", "errors.platform.removal_failed"),
        }
    }
}

/// Chemin d'un élément 🟡 vers la corbeille (ADR 0018).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashRoute {
    /// Déplacement par descripteur dans la quarantaine, puis corbeille de la quarantaine (Linux).
    Quarantine,
    /// Corbeille de l'élément en place, pendant que les dossiers du chemin restent verrouillés (Windows).
    InPlace,
}
