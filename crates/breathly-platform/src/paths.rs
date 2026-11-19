//! Port `StandardPaths` : dossiers standard de l'utilisateur et de Breathly (ADR 0003, ADR 0012).
//!
//! Les dossiers de l'utilisateur sont lus ici pour que les garde-fous S5 et la résolution des chemins des règles
//! n'aient jamais à deviner un emplacement. Un chemin renvoyé est absolu ; un chemin qu'on ne peut pas établir est
//! une erreur, jamais un repli sur un autre dossier.

use std::path::PathBuf;

use crate::error::PlatformError;

/// Dossier de l'utilisateur protégé par les garde-fous S5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserFolder {
    Documents,
    Pictures,
    Videos,
    Music,
    Desktop,
}

impl UserFolder {
    pub(crate) const ALL: [Self; 5] = [
        Self::Documents,
        Self::Pictures,
        Self::Videos,
        Self::Music,
        Self::Desktop,
    ];

    /// Nom anglais du dossier, celui des faux et des tests.
    #[cfg(any(test, feature = "fake"))]
    pub(crate) const fn english_name(self) -> &'static str {
        match self {
            Self::Documents => "Documents",
            Self::Pictures => "Pictures",
            Self::Videos => "Videos",
            Self::Music => "Music",
            Self::Desktop => "Desktop",
        }
    }
}

/// Dossier qu'un adapter n'a pas pu établir ou refuse de renvoyer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardPath {
    Config,
    State,
    Home,
    Temp,
    Folder(UserFolder),
}

pub trait StandardPaths: Send + Sync {
    /// Dossier des réglages de Breathly : `$XDG_CONFIG_HOME/breathly` ou `%APPDATA%\Breathly` (ADR 0003).
    ///
    /// # Errors
    /// `StandardPathUnavailable` si la variable et le repli de l'OS manquent.
    fn config_dir(&self) -> Result<PathBuf, PlatformError>;

    /// Dossier de l'historique : `$XDG_STATE_HOME/breathly` ou `%LOCALAPPDATA%\Breathly` (ADR 0003).
    ///
    /// # Errors
    /// `StandardPathUnavailable` si la variable et le repli de l'OS manquent.
    fn state_dir(&self) -> Result<PathBuf, PlatformError>;

    /// Dossier personnel de l'utilisateur.
    ///
    /// # Errors
    /// `StandardPathUnavailable` si l'OS n'en donne pas.
    fn home_dir(&self) -> Result<PathBuf, PlatformError>;

    /// Dossier des fichiers temporaires que la racine `%TEMP%` d'une règle désigne : `TEMP` (puis `TMP` s'il manque)
    /// sous Windows, `TMPDIR` absolu puis `/tmp` sous Linux. Ne retombe jamais sur le dossier personnel : une règle
    /// qui nettoie toute cette racine viserait tout le profil.
    ///
    /// # Errors
    /// `TempDirUnsafe` (échec fermé) si le dossier est absent, relatif, introuvable, s'il contient `.` ou `..`, si le
    /// dossier personnel est inconnu, s'il contient le dossier personnel ou un dossier protégé, ou s'il est dans un
    /// dossier protégé ; chemins comparés tels qu'écrits puis résolus (liens, noms courts Windows).
    fn temp_dir(&self) -> Result<PathBuf, PlatformError>;

    /// Un des dossiers de l'utilisateur protégés par S5 (`Documents`, `Images`, etc.).
    ///
    /// # Errors
    /// `StandardPathUnavailable` si l'OS ne le connaît pas.
    fn user_folder(&self, folder: UserFolder) -> Result<PathBuf, PlatformError>;
}
