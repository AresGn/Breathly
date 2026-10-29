//! Chemins d'une règle : une racine autorisée suivie de noms simples, sans motif ni remontée.
//!
//! Le contrôle ici est statique (le texte de la règle). Les garde-fous S5 qui dépendent du disque (profil de
//! navigateur, conteneur chiffré, fichier ouvert, lien symbolique) sont revérifiés pour chaque élément au moment
//! d'agir, dans `breathly-disk`.

use crate::rule::Os;

/// Dossiers de l'utilisateur protégés (S5), en minuscules, en français et en anglais.
const PROTECTED_USER_FOLDERS: [&str; 9] = [
    "documents",
    "images",
    "pictures",
    "vidéos",
    "videos",
    "musique",
    "music",
    "bureau",
    "desktop",
];
const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '\\', ':', '*', '?', '<', '>', '"', '|'];

/// Racine autorisée d'un chemin de règle. Tout chemin commence par l'une d'elles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathRoot {
    /// `~` : dossier personnel (Linux).
    Home,
    /// `%LOCALAPPDATA%` (Windows).
    LocalAppData,
    /// `%APPDATA%` (Windows).
    AppData,
    /// `%TEMP%` (Windows).
    Temp,
    /// `%USERPROFILE%` : dossier personnel (Windows).
    UserProfile,
}

impl PathRoot {
    const ALL: [Self; 5] = [
        Self::Home,
        Self::LocalAppData,
        Self::AppData,
        Self::Temp,
        Self::UserProfile,
    ];

    /// Forme écrite dans le fichier de règles.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Home => "~",
            Self::LocalAppData => "%LOCALAPPDATA%",
            Self::AppData => "%APPDATA%",
            Self::Temp => "%TEMP%",
            Self::UserProfile => "%USERPROFILE%",
        }
    }

    #[must_use]
    pub const fn os(self) -> Os {
        match self {
            Self::Home => Os::Linux,
            Self::LocalAppData | Self::AppData | Self::Temp | Self::UserProfile => Os::Windows,
        }
    }

    /// Racine qui contient `Documents`, `Images`, etc. directement.
    const fn holds_user_folders(self) -> bool {
        match self {
            Self::Home | Self::UserProfile => true,
            Self::LocalAppData | Self::AppData | Self::Temp => false,
        }
    }
}

/// Chemin de règle validé : une racine et des noms simples (au moins un, sauf pour un dossier de recherche).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RulePath {
    root: PathRoot,
    components: Vec<String>,
}

impl RulePath {
    #[must_use]
    pub const fn root(&self) -> PathRoot {
        self.root
    }

    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

/// Pourquoi un chemin ou un nom est refusé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathRefusal {
    /// Ne commence pas par une racine autorisée (chemin absolu, racine du disque, variable inconnue).
    OutsideAllowedRoots,
    /// Racine d'un autre système que celui de la liste (`%TEMP%` dans `paths.linux`).
    RootOfOtherOs,
    /// La racine seule : un dossier personnel entier n'est jamais une cible.
    RootItself,
    /// Nom vide, `.` ou `..` : le chemin doit être écrit sous sa forme simple.
    NotNormalized,
    /// Séparateur (`\` partout, `/` dans un nom seul), motif (`*`, `?`) ou caractère interdit par Windows.
    ForbiddenCharacter,
    /// Dossier protégé de l'utilisateur (S5).
    ProtectedFolder,
    /// Dossier `.git` (S5).
    GitFolder,
}

/// Place d'un chemin dans la règle : cible nettoyée, ou dossier où l'on cherche des dossiers de projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    Cleaned,
    SearchRoot,
}

pub(crate) fn parse_rule_path(
    raw: &str,
    os: Os,
    placement: Placement,
) -> Result<RulePath, PathRefusal> {
    let mut parts = raw.split('/');
    let prefix = parts.next().unwrap_or_default();
    let root = PathRoot::ALL
        .into_iter()
        .find(|root| root.token() == prefix)
        .ok_or(PathRefusal::OutsideAllowedRoots)?;
    if root.os() != os {
        return Err(PathRefusal::RootOfOtherOs);
    }
    let components: Vec<String> = parts.map(str::to_owned).collect();
    components
        .iter()
        .try_for_each(|name| check_component(name))?;
    if components.is_empty() && placement == Placement::Cleaned {
        return Err(PathRefusal::RootItself);
    }
    let is_user_folder = components
        .first()
        .is_some_and(|first| is_protected_folder(first));
    if root.holds_user_folders() && is_user_folder {
        return Err(PathRefusal::ProtectedFolder);
    }
    Ok(RulePath { root, components })
}

/// Nom d'un seul élément (dossier ou fichier marqueur d'une règle « dossier de projet »).
pub(crate) fn check_name(name: &str) -> Result<(), PathRefusal> {
    check_component(name)?;
    if is_protected_folder(name) {
        return Err(PathRefusal::ProtectedFolder);
    }
    Ok(())
}

fn check_component(name: &str) -> Result<(), PathRefusal> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(PathRefusal::NotNormalized);
    }
    if name
        .chars()
        .any(|c| c.is_control() || FORBIDDEN_CHARACTERS.contains(&c))
    {
        return Err(PathRefusal::ForbiddenCharacter);
    }
    if name.eq_ignore_ascii_case(".git") {
        return Err(PathRefusal::GitFolder);
    }
    Ok(())
}

fn is_protected_folder(name: &str) -> bool {
    PROTECTED_USER_FOLDERS.contains(&name.to_lowercase().as_str())
}
