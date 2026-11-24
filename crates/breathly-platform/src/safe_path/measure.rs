//! Échec de la mesure d'un élément relative à une ancre. Aucune variante ne porte de chemin.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MeasureError {
    #[error("l'élément contient un autre volume")]
    CrossVolume,
    #[error("la mesure a échoué")]
    Failed,
}

impl MeasureError {
    /// Code stable, repris dans l'historique et les journaux.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::CrossVolume => "platform.measure_cross_volume",
            Self::Failed => "platform.measure_failed",
        }
    }

    /// Clé du texte affiché, dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(self) -> &'static str {
        match self {
            Self::CrossVolume => "errors.platform.measure_cross_volume",
            Self::Failed => "errors.platform.measure_failed",
        }
    }
}
