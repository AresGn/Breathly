//! Échecs du nettoyage et de sa simulation : un code stable et une clé de traduction, jamais un chemin (SEC9).

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CleanError {
    #[error("jeton de confirmation invalide")]
    TokenInvalid,
    #[error("jeton de confirmation expiré")]
    TokenExpired,
    #[error("jeton de confirmation déjà utilisé")]
    TokenUsed,
    #[error("aucun élément sélectionné")]
    EmptySelection,
    #[error("aléa indisponible pour le jeton")]
    EntropyUnavailable,
    #[error("horloge inutilisable")]
    ClockRange,
}

impl CleanError {
    /// Code stable, repris dans les journaux et par l'interface.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::TokenInvalid => "clean.token_invalid",
            Self::TokenExpired => "clean.token_expired",
            Self::TokenUsed => "clean.token_used",
            Self::EmptySelection => "clean.empty_selection",
            Self::EntropyUnavailable => "clean.entropy_unavailable",
            Self::ClockRange => "clean.clock_range",
        }
    }

    /// Clé du texte affiché, dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(self) -> &'static str {
        match self {
            Self::TokenInvalid => "errors.clean.token_invalid",
            Self::TokenExpired => "errors.clean.token_expired",
            Self::TokenUsed => "errors.clean.token_used",
            Self::EmptySelection => "errors.clean.empty_selection",
            Self::EntropyUnavailable => "errors.clean.entropy_unavailable",
            Self::ClockRange => "errors.clean.clock_range",
        }
    }
}
