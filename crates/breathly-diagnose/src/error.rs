//! Erreurs de `breathly-diagnose` : un code stable et une clé de traduction par variante (CODING.md §5).

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DiagnoseError {
    #[error("fenêtre de mesure nulle")]
    EmptySamplingWindow,
}

impl DiagnoseError {
    /// Code stable, repris dans les journaux et par l'interface.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptySamplingWindow => "diagnose.empty_sampling_window",
        }
    }

    /// Clé du texte affiché, dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        match self {
            Self::EmptySamplingWindow => "errors.diagnose.empty_sampling_window",
        }
    }
}
