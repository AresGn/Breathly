//! Contrôle du dossier temporaire : il ne doit jamais contenir le profil de l'utilisateur ni un dossier protégé
//! (S5), ni se trouver dans un dossier protégé. Tout chemin qu'on ne sait pas lire sans ambiguïté est refusé.

use std::path::{Path, PathBuf};

/// Façon d'écrire et de comparer les chemins : séparateurs et casse changent d'un système à l'autre. La logique
/// Windows se teste sous Linux, d'où ce type plutôt qu'un `cfg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathStyle {
    Posix,
    Windows,
}

const WINDOWS_VERBATIM: &str = r"\\?\";
const WINDOWS_DEVICE: &str = r"\\.\";

impl PathStyle {
    pub(crate) fn is_absolute(self, path: &Path) -> bool {
        match self {
            Self::Posix => path.to_string_lossy().starts_with('/'),
            Self::Windows => is_windows_absolute(&path.to_string_lossy()),
        }
    }

    /// Noms d'un chemin absolu, ou `None` s'il est relatif ou ambigu (`.`, `..`, espace de noms des périphériques).
    /// Sous Windows : casse ignorée, préfixe `\\?\` retiré, points et espaces finaux retirés comme le fait Win32.
    fn names(self, path: &Path) -> Option<Vec<String>> {
        let text = path.to_string_lossy();
        match self {
            Self::Posix => text
                .starts_with('/')
                .then(|| plain_names(text.split('/')))?,
            Self::Windows => windows_names(&text.replace('/', "\\")),
        }
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

fn plain_names<'a>(parts: impl Iterator<Item = &'a str>) -> Option<Vec<String>> {
    parts
        .filter(|name| !name.is_empty())
        .map(|name| (name != "." && name != "..").then(|| name.to_owned()))
        .collect()
}

/// `\\?\C:\x` devient `C:\x`, `\\?\UNC\srv\x` devient `\\srv\x` ; `\\.\` (périphériques) est refusé.
fn windows_names(text: &str) -> Option<Vec<String>> {
    if text.starts_with(WINDOWS_DEVICE) {
        return None;
    }
    let text = match text.strip_prefix(WINDOWS_VERBATIM) {
        Some(rest)
            if rest
                .get(..4)
                .is_some_and(|unc| unc.eq_ignore_ascii_case("UNC\\")) =>
        {
            format!("\\\\{}", &rest[4..])
        }
        Some(rest) => rest.to_owned(),
        None => text.to_owned(),
    };
    if !is_windows_absolute(&text) || text.contains('?') {
        return None;
    }
    let trimmed = text
        .split('\\')
        .filter(|name| !name.is_empty())
        .map(|name| match name.trim_end_matches(['.', ' ']) {
            "" => "..",
            cut => cut,
        });
    let names = plain_names(trimmed)?;
    let has_stream = names.iter().skip(1).any(|name| name.contains(':'));
    (!has_stream).then(|| names.iter().map(|name| name.to_lowercase()).collect())
}

/// Dossiers que le dossier temporaire ne doit jamais englober : le profil, et les dossiers de l'utilisateur dans
/// lesquels il ne doit pas non plus se trouver. Chaque chemin y figure tel qu'écrit et, si on l'a pu, résolu.
#[derive(Debug, Default)]
pub(crate) struct Protected {
    pub(crate) profiles: Vec<PathBuf>,
    pub(crate) folders: Vec<PathBuf>,
}

/// Le dossier temporaire est sûr au regard des dossiers protégés. Échec fermé : profil inconnu ou illisible, refus.
pub(crate) fn is_safe_temp(temp: &Path, protected: &Protected, style: PathStyle) -> bool {
    let Some(temp) = style.names(temp).filter(|names| !names.is_empty()) else {
        return false;
    };
    let names_of = |paths: &[PathBuf]| -> Option<Vec<Vec<String>>> {
        paths.iter().map(|path| style.names(path)).collect()
    };
    let (Some(profiles), Some(folders)) =
        (names_of(&protected.profiles), names_of(&protected.folders))
    else {
        return false;
    };
    let holds_a_protected_folder = profiles
        .iter()
        .chain(&folders)
        .any(|inner| inner.starts_with(&temp));
    let is_in_a_user_folder = folders.iter().any(|folder| temp.starts_with(folder));
    !profiles.is_empty() && !holds_a_protected_folder && !is_in_a_user_folder
}

#[cfg(test)]
mod tests;
