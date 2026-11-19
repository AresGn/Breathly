//! Adapter réel de `StandardPaths` : variables d'environnement et dossiers connus de l'OS (crate `dirs`).
//!
//! La composition des chemins ne lit rien elle-même : elle passe par `Sources`, ce qui permet de la tester avec
//! des valeurs choisies, pour Linux comme pour Windows, sans toucher au profil de la personne qui lance les tests.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::PlatformError;
use crate::paths::{StandardPath, StandardPaths, UserFolder};
use crate::temp_guard::{PathStyle, Protected, is_safe_temp};

const POSIX_APP_FOLDER: &str = "breathly";
const WINDOWS_APP_FOLDER: &str = "Breathly";
const POSIX_TEMP_FALLBACK: &str = "/tmp";

/// Ce que l'adapter lit dans le système.
pub(crate) trait Sources: Send + Sync {
    fn var(&self, name: &str) -> Option<OsString>;
    fn home(&self) -> Option<PathBuf>;
    fn user_folder(&self, folder: UserFolder) -> Option<PathBuf>;
    /// Chemin réel : liens suivis, noms courts Windows développés ; `None` s'il n'existe pas.
    fn resolve(&self, path: &Path) -> Option<PathBuf>;
}

struct OsSources;

impl Sources for OsSources {
    fn var(&self, name: &str) -> Option<OsString> {
        env::var_os(name)
    }

    fn home(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn user_folder(&self, folder: UserFolder) -> Option<PathBuf> {
        match folder {
            UserFolder::Documents => dirs::document_dir(),
            UserFolder::Pictures => dirs::picture_dir(),
            UserFolder::Videos => dirs::video_dir(),
            UserFolder::Music => dirs::audio_dir(),
            UserFolder::Desktop => dirs::desktop_dir(),
        }
    }

    fn resolve(&self, path: &Path) -> Option<PathBuf> {
        fs::canonicalize(path).ok()
    }
}

/// Les dossiers standard du système sur lequel Breathly tourne.
pub struct SystemStandardPaths {
    style: PathStyle,
    sources: Box<dyn Sources>,
}

impl SystemStandardPaths {
    #[must_use]
    pub fn new() -> Self {
        let style = if cfg!(windows) {
            PathStyle::Windows
        } else {
            PathStyle::Posix
        };
        Self::with_sources(style, Box::new(OsSources))
    }

    pub(crate) fn with_sources(style: PathStyle, sources: Box<dyn Sources>) -> Self {
        Self { style, sources }
    }

    fn non_empty_var(&self, name: &str) -> Option<PathBuf> {
        self.sources
            .var(name)
            .filter(|raw| !raw.is_empty())
            .map(PathBuf::from)
    }

    /// Variable d'environnement qui doit contenir un chemin absolu, sinon elle est ignorée (spec XDG).
    fn absolute_var(&self, name: &str) -> Option<PathBuf> {
        self.non_empty_var(name)
            .filter(|value| self.style.is_absolute(value))
    }

    fn home(&self) -> Result<PathBuf, PlatformError> {
        self.sources
            .home()
            .filter(|home| self.style.is_absolute(home))
            .ok_or(unavailable(StandardPath::Home))
    }

    /// Dossier de l'application sous la variable de l'OS ; sous Linux, sous le défaut XDG du dossier personnel.
    fn app_dir(&self, kind: AppDir) -> Result<PathBuf, PlatformError> {
        let (base, folder) = match self.style {
            PathStyle::Posix => (
                self.absolute_var(kind.xdg_variable()).or_else(|| {
                    let home = self.home().ok()?;
                    Some(
                        kind.xdg_fallback()
                            .iter()
                            .fold(home, |path, name| path.join(name)),
                    )
                }),
                POSIX_APP_FOLDER,
            ),
            PathStyle::Windows => (
                self.absolute_var(kind.windows_variable()),
                WINDOWS_APP_FOLDER,
            ),
        };
        base.map(|base| base.join(folder))
            .ok_or(unavailable(kind.standard_path()))
    }

    /// Windows : `TEMP`, puis `TMP` s'il manque (jamais si `TEMP` est présent mais refusé). Linux : `TMPDIR`
    /// absolu, sinon `/tmp`.
    fn temp_candidate(&self) -> Option<PathBuf> {
        match self.style {
            PathStyle::Windows => self
                .non_empty_var("TEMP")
                .or_else(|| self.non_empty_var("TMP")),
            PathStyle::Posix => self
                .absolute_var("TMPDIR")
                .or_else(|| Some(PathBuf::from(POSIX_TEMP_FALLBACK))),
        }
    }

    /// Profil et dossiers protégés, tels qu'écrits et résolus.
    fn protected(&self) -> Protected {
        let profile_var = match self.style {
            PathStyle::Windows => "USERPROFILE",
            PathStyle::Posix => "HOME",
        };
        let profiles = [self.sources.home(), self.non_empty_var(profile_var)];
        let folders = UserFolder::ALL.map(|folder| self.sources.user_folder(folder));
        Protected {
            profiles: self.with_resolved(profiles.into_iter().flatten()),
            folders: self.with_resolved(folders.into_iter().flatten()),
        }
    }

    fn with_resolved(&self, paths: impl Iterator<Item = PathBuf>) -> Vec<PathBuf> {
        paths
            .flat_map(|path| [self.sources.resolve(&path), Some(path)])
            .flatten()
            .collect()
    }
}

impl Default for SystemStandardPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum AppDir {
    Config,
    State,
}

impl AppDir {
    const fn standard_path(self) -> StandardPath {
        match self {
            Self::Config => StandardPath::Config,
            Self::State => StandardPath::State,
        }
    }

    const fn xdg_variable(self) -> &'static str {
        match self {
            Self::Config => "XDG_CONFIG_HOME",
            Self::State => "XDG_STATE_HOME",
        }
    }

    const fn xdg_fallback(self) -> &'static [&'static str] {
        match self {
            Self::Config => &[".config"],
            Self::State => &[".local", "state"],
        }
    }

    const fn windows_variable(self) -> &'static str {
        match self {
            Self::Config => "APPDATA",
            Self::State => "LOCALAPPDATA",
        }
    }
}

const fn unavailable(path: StandardPath) -> PlatformError {
    PlatformError::StandardPathUnavailable { path }
}

impl StandardPaths for SystemStandardPaths {
    fn config_dir(&self) -> Result<PathBuf, PlatformError> {
        self.app_dir(AppDir::Config)
    }

    fn state_dir(&self) -> Result<PathBuf, PlatformError> {
        self.app_dir(AppDir::State)
    }

    fn home_dir(&self) -> Result<PathBuf, PlatformError> {
        self.home()
    }

    /// Le chemin est jugé tel qu'écrit puis résolu : un lien ou un nom court vers le profil est refusé.
    fn temp_dir(&self) -> Result<PathBuf, PlatformError> {
        let temp = self.temp_candidate().ok_or(PlatformError::TempDirUnsafe)?;
        let resolved = self
            .sources
            .resolve(&temp)
            .ok_or(PlatformError::TempDirUnsafe)?;
        let protected = self.protected();
        let is_safe = [&temp, &resolved]
            .iter()
            .all(|path| is_safe_temp(path, &protected, self.style));
        is_safe.then_some(temp).ok_or(PlatformError::TempDirUnsafe)
    }

    fn user_folder(&self, folder: UserFolder) -> Result<PathBuf, PlatformError> {
        self.sources
            .user_folder(folder)
            .filter(|path| self.style.is_absolute(path))
            .ok_or(unavailable(StandardPath::Folder(folder)))
    }
}

#[cfg(test)]
mod tests;
