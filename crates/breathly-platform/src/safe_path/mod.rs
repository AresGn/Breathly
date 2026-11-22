//! Chemins canoniques et ancrage sans suivre de lien (ADR 0018, ADR 0019).
//!
//! Deux types portent la règle : [`CanonicalPath`] (seul moyen de comparer des chemins : les deux côtés sont
//! canonicalisés) et [`PathAnchor`] (l'identité d'un élément, obtenue en ouvrant chaque dossier de la chaîne sans
//! suivre de lien). Aucun autre crate ne compare des chemins à la main.

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as os;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as os;

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

use crate::error::PlatformError;

/// Pourquoi un élément ne peut pas être ancré : un refus sans perte, jamais une suppression de repli.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AnchorRefusal {
    /// Un composant du chemin est devenu un lien, une jonction, ou a disparu (`path_changed`).
    #[error("un dossier du chemin a changé depuis la vérification")]
    PathChanged,
    /// Un dossier du chemin ne peut pas être ouvert : utilisé par un autre programme ou refusé (`path_busy`).
    #[error("un dossier du chemin est utilisé par un autre programme")]
    PathBusy,
    /// Un dossier du chemin est refusé par les permissions (`path_denied`).
    #[error("un dossier du chemin est inaccessible (permissions)")]
    PathDenied,
}

/// Chemin absolu passé par `fs::canonicalize` : liens suivis, casse et noms courts Windows normalisés.
///
/// Un chemin brut ne se compare jamais à un chemin canonique (ADR 0019) : le type l'interdit. Son `Debug` masque
/// la valeur (SEC9).
#[derive(Clone)]
pub struct CanonicalPath(PathBuf);

impl fmt::Debug for CanonicalPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CanonicalPath(..)")
    }
}

/// Égalité composante par composante, comme `is_within` : sans tenir compte de la casse sous Windows.
impl PartialEq for CanonicalPath {
    fn eq(&self, other: &Self) -> bool {
        self.is_within(other) && other.is_within(self)
    }
}

impl Eq for CanonicalPath {}

impl CanonicalPath {
    /// Canonicalise `path`.
    ///
    /// # Errors
    /// `PathUnresolvable` si le chemin n'existe pas ou ne peut pas être résolu : l'élément est refusé.
    pub fn resolve(path: &Path) -> Result<Self, PlatformError> {
        fs::canonicalize(path)
            .map(Self)
            .map_err(|_| PlatformError::PathUnresolvable)
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Le chemin à montrer à l'utilisateur : sans le préfixe `\\?\` de Windows, qui n'est retiré qu'ici (ADR 0019).
    #[must_use]
    pub fn display_path(&self) -> PathBuf {
        let text = self.0.to_string_lossy();
        match text.strip_prefix(r"\\?\") {
            Some(rest) if !rest.starts_with("UNC\\") => PathBuf::from(rest),
            Some(_) | None => self.0.clone(),
        }
    }

    /// Vrai si `self` est `root` ou se trouve dessous (comparaison composante par composante, sans tenir compte de
    /// la casse sous Windows).
    #[must_use]
    pub fn is_within(&self, root: &Self) -> bool {
        self.components_below(root).is_some()
    }

    fn components_below(&self, root: &Self) -> Option<Vec<&OsStr>> {
        let mut own = self.0.components();
        for expected in root.0.components() {
            let found = own.next()?;
            if !same_component(found, expected) {
                return None;
            }
        }
        Some(own.map(Component::as_os_str).collect())
    }
}

fn same_component(left: Component<'_>, right: Component<'_>) -> bool {
    if cfg!(windows) {
        left.as_os_str().to_string_lossy().to_lowercase()
            == right.as_os_str().to_string_lossy().to_lowercase()
    } else {
        left == right
    }
}

/// Identité d'un élément : les dossiers de sa chaîne, tenus ouverts sans suivre de lien (ADR 0018).
///
/// Linux : descripteur du dossier parent et nom, chaque niveau ouvert avec `O_NOFOLLOW`. Windows : handles des
/// dossiers de la chaîne, sans partage de suppression, aucun n'étant un point d'analyse. Les opérations de
/// suppression (tickets #81, #82) s'appuient sur ce type et jamais sur un chemin. Son `Debug` masque la valeur.
pub struct PathAnchor {
    held: os::Held,
}

impl fmt::Debug for PathAnchor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PathAnchor(..)")
    }
}

impl PathAnchor {
    /// Ouvre la chaîne de dossiers de `root` à `target`, sans suivre de lien, et vérifie que l'élément existe.
    ///
    /// # Errors
    /// `PathChanged` si `target` n'est pas sous `root`, ou si un composant est devenu un lien, une jonction ou a
    /// disparu ; `PathBusy` si un dossier est utilisé par un autre programme ; `PathDenied` si les permissions l'interdisent.
    pub fn open(root: &CanonicalPath, target: &CanonicalPath) -> Result<Self, AnchorRefusal> {
        let below = target
            .components_below(root)
            .filter(|names| !names.is_empty())
            .ok_or(AnchorRefusal::PathChanged)?;
        os::hold(root.as_path(), &below).map(|held| Self { held })
    }

    /// Nom de l'élément dans son dossier parent.
    #[must_use]
    pub fn file_name(&self) -> &OsStr {
        self.held.file_name()
    }
}
