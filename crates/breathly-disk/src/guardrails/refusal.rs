//! Les refus des garde-fous : un résultat typé, jamais une erreur du nettoyage (design 0001 : l'élément est
//! « ignoré » et les autres continuent).

use breathly_platform::{AnchorRefusal, PlatformError};
use thiserror::Error;

/// Pourquoi un élément n'est pas vérifié. Aucune variante ne porte de chemin (SEC9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Refusal {
    #[error("un élément 🔴 n'est jamais sélectionnable")]
    NotSelectable,
    #[error("l'élément n'est dans aucun emplacement de la règle")]
    OutsideRule,
    #[error("dossier personnel protégé (Documents, Images, Vidéos, Musique, Bureau)")]
    UserFolder,
    #[error("profil de navigateur")]
    BrowserProfile,
    #[error("conteneur chiffré")]
    EncryptedContainer,
    #[error("dossier .git")]
    GitFolder,
    #[error("chemin introuvable ou impossible à résoudre")]
    PathUnresolvable,
    #[error("un dossier du chemin a changé depuis l'analyse")]
    PathChanged,
    #[error("un dossier du chemin est utilisé par un autre programme")]
    PathBusy,
    #[error("un dossier du chemin est inaccessible (permissions)")]
    PathDenied,
}

impl Refusal {
    /// Code stable, repris dans l'historique et les journaux.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.names().0
    }

    /// Clé du texte d'explication affiché (US-036.4), dans les catalogues de `locales/`.
    #[must_use]
    pub const fn translation_key(self) -> &'static str {
        self.names().1
    }

    const fn names(self) -> (&'static str, &'static str) {
        match self {
            Self::NotSelectable => ("guard.not_selectable", "errors.guard.not_selectable"),
            Self::OutsideRule => ("guard.outside_rule", "errors.guard.outside_rule"),
            Self::UserFolder => ("guard.user_folder", "errors.guard.user_folder"),
            Self::BrowserProfile => ("guard.browser_profile", "errors.guard.browser_profile"),
            Self::EncryptedContainer => (
                "guard.encrypted_container",
                "errors.guard.encrypted_container",
            ),
            Self::GitFolder => ("guard.git_folder", "errors.guard.git_folder"),
            Self::PathUnresolvable => ("guard.path_unresolvable", "errors.guard.path_unresolvable"),
            Self::PathChanged => ("guard.path_changed", "errors.guard.path_changed"),
            Self::PathBusy => ("guard.path_busy", "errors.guard.path_busy"),
            Self::PathDenied => ("guard.path_denied", "errors.guard.path_denied"),
        }
    }
}

impl From<AnchorRefusal> for Refusal {
    fn from(refusal: AnchorRefusal) -> Self {
        match refusal {
            AnchorRefusal::PathChanged => Self::PathChanged,
            AnchorRefusal::PathBusy => Self::PathBusy,
            AnchorRefusal::PathDenied => Self::PathDenied,
        }
    }
}

/// Un chemin que la plateforme ne sait pas résoudre est refusé (échec fermé).
impl From<PlatformError> for Refusal {
    fn from(_: PlatformError) -> Self {
        Self::PathUnresolvable
    }
}
