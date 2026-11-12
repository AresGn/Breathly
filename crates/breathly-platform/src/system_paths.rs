//! Adapter réel de `StandardPaths` : variables d'environnement et dossiers connus de l'OS (crate `dirs`).
//!
//! La composition des chemins ne lit rien elle-même : elle passe par `Sources`, ce qui permet de la tester avec
//! des valeurs choisies, pour Linux comme pour Windows, sans toucher au profil de la personne qui lance les tests.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::PlatformError;
use crate::paths::{StandardPath, StandardPaths, UserFolder};
use crate::temp_guard::{PathStyle, safe_temp};

const POSIX_APP_FOLDER: &str = "breathly";
const WINDOWS_APP_FOLDER: &str = "Breathly";
const POSIX_TEMP_FALLBACK: &str = "/tmp";

/// Ce que l'adapter lit dans le système.
pub(crate) trait Sources: Send + Sync {
    fn var(&self, name: &str) -> Option<OsString>;
    fn home(&self) -> Option<PathBuf>;
    fn user_folder(&self, folder: UserFolder) -> Option<PathBuf>;
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
}

pub(crate) struct ComposedPaths<S> {
    style: PathStyle,
    sources: S,
}

impl<S: Sources> ComposedPaths<S> {
    pub(crate) const fn new(style: PathStyle, sources: S) -> Self {
        Self { style, sources }
    }

    /// Variable d'environnement qui doit contenir un chemin absolu, sinon elle est ignorée (spec XDG).
    fn absolute_var(&self, name: &str) -> Option<PathBuf> {
        let value = PathBuf::from(self.sources.var(name).filter(|raw| !raw.is_empty())?);
        self.style.is_absolute(&value).then_some(value)
    }

    fn home(&self) -> Result<PathBuf, PlatformError> {
        self.sources
            .home()
            .filter(|home| self.style.is_absolute(home))
            .ok_or(unavailable(StandardPath::Home))
    }

    /// Dossier de l'application sous `variable`, ou sous `~/fallback` quand elle manque (Linux seulement).
    fn app_dir(&self, kind: AppDir) -> Result<PathBuf, PlatformError> {
        let which = kind.standard_path();
        let base = match self.style {
            PathStyle::Posix => self
                .absolute_var(kind.xdg_variable())
                .or_else(|| self.home().ok().map(|home| home.join(kind.xdg_fallback()))),
            PathStyle::Windows => self.absolute_var(kind.windows_variable()),
        };
        let folder = match self.style {
            PathStyle::Posix => POSIX_APP_FOLDER,
            PathStyle::Windows => WINDOWS_APP_FOLDER,
        };
        base.map(|base| base.join(folder)).ok_or(unavailable(which))
    }

    fn temp_candidate(&self) -> Option<OsString> {
        match self.style {
            PathStyle::Windows => self.sources.var("TEMP"),
            PathStyle::Posix => self
                .sources
                .var("TMPDIR")
                .filter(|raw| !raw.is_empty())
                .or_else(|| Some(OsString::from(POSIX_TEMP_FALLBACK))),
        }
    }

    fn profiles(&self) -> Vec<PathBuf> {
        let userprofile = self.sources.var("USERPROFILE").map(PathBuf::from);
        [self.sources.home(), userprofile]
            .into_iter()
            .flatten()
            .collect()
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

    const fn xdg_fallback(self) -> &'static str {
        match self {
            Self::Config => ".config",
            Self::State => ".local/state",
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

impl<S: Sources> StandardPaths for ComposedPaths<S> {
    fn config_dir(&self) -> Result<PathBuf, PlatformError> {
        self.app_dir(AppDir::Config)
    }

    fn state_dir(&self) -> Result<PathBuf, PlatformError> {
        self.app_dir(AppDir::State)
    }

    fn home_dir(&self) -> Result<PathBuf, PlatformError> {
        self.home()
    }

    fn temp_dir(&self) -> Result<PathBuf, PlatformError> {
        let candidate = self.temp_candidate();
        safe_temp(candidate.as_deref(), &self.profiles(), self.style)
            .ok_or(PlatformError::TempDirUnsafe)
    }

    fn user_folder(&self, folder: UserFolder) -> Result<PathBuf, PlatformError> {
        self.sources
            .user_folder(folder)
            .filter(|path| self.style.is_absolute(path))
            .ok_or(unavailable(StandardPath::Folder(folder)))
    }
}

/// Les dossiers standard du système sur lequel Breathly tourne.
pub struct SystemStandardPaths(ComposedPaths<OsSources>);

impl SystemStandardPaths {
    #[must_use]
    pub const fn new() -> Self {
        let style = if cfg!(windows) {
            PathStyle::Windows
        } else {
            PathStyle::Posix
        };
        Self(ComposedPaths::new(style, OsSources))
    }
}

impl Default for SystemStandardPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardPaths for SystemStandardPaths {
    fn config_dir(&self) -> Result<PathBuf, PlatformError> {
        self.0.config_dir()
    }

    fn state_dir(&self) -> Result<PathBuf, PlatformError> {
        self.0.state_dir()
    }

    fn home_dir(&self) -> Result<PathBuf, PlatformError> {
        self.0.home_dir()
    }

    fn temp_dir(&self) -> Result<PathBuf, PlatformError> {
        self.0.temp_dir()
    }

    fn user_folder(&self, folder: UserFolder) -> Result<PathBuf, PlatformError> {
        self.0.user_folder(folder)
    }
}

#[cfg(test)]
mod tests;
