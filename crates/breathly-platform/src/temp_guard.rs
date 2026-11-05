//! Contrôle du dossier temporaire : il ne doit jamais désigner le profil de l'utilisateur ni l'un de ses parents.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Façon d'écrire et de comparer les chemins : séparateurs et casse changent d'un système à l'autre. La logique
/// Windows se teste sous Linux, d'où ce type plutôt qu'un `cfg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathStyle {
    Posix,
    Windows,
}

impl PathStyle {
    pub(crate) fn is_absolute(self, path: &Path) -> bool {
        match self {
            Self::Posix => path.to_string_lossy().starts_with('/'),
            Self::Windows => is_windows_absolute(&path.to_string_lossy()),
        }
    }

    /// Noms du chemin, sans séparateur ni élément vide ; en minuscules sous Windows, qui ignore la casse.
    fn names(self, path: &Path) -> Vec<String> {
        let text = path.to_string_lossy();
        match self {
            Self::Posix => text
                .split('/')
                .filter(|n| !n.is_empty())
                .map(str::to_owned)
                .collect(),
            Self::Windows => text
                .split(['/', '\\'])
                .filter(|n| !n.is_empty())
                .map(str::to_lowercase)
                .collect(),
        }
    }

    /// `ancestor` est `path` ou l'un de ses parents.
    fn contains(self, ancestor: &Path, path: &Path) -> bool {
        let (ancestor, path) = (self.names(ancestor), self.names(path));
        path.starts_with(&ancestor)
    }
}

fn is_windows_absolute(text: &str) -> bool {
    let mut chars = text.chars();
    let is_drive = matches!(
        (chars.next(), chars.next(), chars.next()),
        (Some(letter), Some(':'), Some('/' | '\\')) if letter.is_ascii_alphabetic()
    );
    is_drive || text.starts_with("\\\\")
}

/// Le dossier temporaire annoncé par l'environnement, ou `None` s'il n'est pas sûr.
pub(crate) fn safe_temp(
    candidate: Option<&OsStr>,
    profiles: &[PathBuf],
    style: PathStyle,
) -> Option<PathBuf> {
    let temp = PathBuf::from(candidate.filter(|raw| !raw.is_empty())?);
    let is_safe = style.is_absolute(&temp)
        && !style.names(&temp).is_empty()
        && !profiles
            .iter()
            .any(|profile| style.contains(&temp, profile));
    is_safe.then_some(temp)
}
