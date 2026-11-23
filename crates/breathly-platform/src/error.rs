//! Erreurs de `breathly-platform` : un code stable et une clé de traduction par variante (CODING.md §5).
//!
//! Une erreur peut finir dans un journal (SEC9) : aucune variante ne porte de chemin en clair, et l'identifiant
//! d'un élément de la corbeille (`TrashItemId`, parfois un chemin) est masqué par son `Debug` et absent du message.

use thiserror::Error;

use crate::paths::StandardPath;
use crate::trash::TrashItemId;

/// Mesure du système qu'un adapter n'a pas pu lire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    Processes,
    Memory,
    Swap,
    Cpu,
    Volumes,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlatformError {
    #[error("mesure indisponible : {measure:?}")]
    MeasureUnavailable { measure: Measure },
    #[error("corbeille du système inutilisable")]
    TrashUnavailable,
    #[error("élément à mettre à la corbeille introuvable")]
    TrashSourceMissing,
    #[error("élément absent de la corbeille")]
    TrashItemNotFound { item: TrashItemId },
    #[error("emplacement d'origine occupé : rien n'a été restauré")]
    RestoreCollision { item: TrashItemId },
    #[error("dossier standard indisponible : {path:?}")]
    StandardPathUnavailable { path: StandardPath },
    #[error("dossier temporaire absent ou confondu avec le dossier personnel")]
    TempDirUnsafe,
    #[error("chemin introuvable ou impossible à résoudre")]
    PathUnresolvable,
    #[error("le système n'a pas fourni d'aléa")]
    EntropyUnavailable,
    #[error("impossible de réserver le fichier à l'utilisateur")]
    OwnerOnlyFailed,
}

impl PlatformError {
    /// Code stable, repris dans les journaux et par l'interface.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MeasureUnavailable { .. } => "platform.measure_unavailable",
            Self::TrashUnavailable => "platform.trash_unavailable",
            Self::TrashSourceMissing => "platform.trash_source_missing",
            Self::TrashItemNotFound { .. } => "platform.trash_item_not_found",
            Self::RestoreCollision { .. } => "platform.restore_collision",
            Self::StandardPathUnavailable { .. } => "platform.standard_path_unavailable",
            Self::TempDirUnsafe => "platform.temp_dir_unsafe",
            Self::PathUnresolvable => "platform.path_unresolvable",
            Self::EntropyUnavailable => "platform.entropy_unavailable",
            Self::OwnerOnlyFailed => "platform.owner_only_failed",
        }
    }

    /// Clé du texte affiché, dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        match self {
            Self::MeasureUnavailable { .. } => "errors.platform.measure_unavailable",
            Self::TrashUnavailable => "errors.platform.trash_unavailable",
            Self::TrashSourceMissing => "errors.platform.trash_source_missing",
            Self::TrashItemNotFound { .. } => "errors.platform.trash_item_not_found",
            Self::RestoreCollision { .. } => "errors.platform.restore_collision",
            Self::StandardPathUnavailable { .. } => "errors.platform.standard_path_unavailable",
            Self::TempDirUnsafe => "errors.platform.temp_dir_unsafe",
            Self::PathUnresolvable => "errors.platform.path_unresolvable",
            Self::EntropyUnavailable => "errors.platform.entropy_unavailable",
            Self::OwnerOnlyFailed => "errors.platform.owner_only_failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_error_has_a_code_and_a_translation_key_built_from_it() {
        let item = TrashItemId::new("id".to_owned());
        let errors = [
            PlatformError::MeasureUnavailable {
                measure: Measure::Cpu,
            },
            PlatformError::TrashUnavailable,
            PlatformError::TrashSourceMissing,
            PlatformError::TrashItemNotFound { item: item.clone() },
            PlatformError::RestoreCollision { item },
            PlatformError::StandardPathUnavailable {
                path: StandardPath::Config,
            },
            PlatformError::TempDirUnsafe,
            PlatformError::PathUnresolvable,
            PlatformError::EntropyUnavailable,
            PlatformError::OwnerOnlyFailed,
        ];
        for error in errors {
            assert!(error.code().starts_with("platform."));
            assert_eq!(error.translation_key(), format!("errors.{}", error.code()));
        }
    }
}
