//! Port `OpenFileInspector` : un élément est-il ouvert par un logiciel ? (ADR 0020, garde-fou S5)
//!
//! Le résultat est typé : « libre », « utilisé par ces logiciels » ou « vérification impossible » ; jamais une
//! erreur du nettoyage. Un élément est un fichier ou un dossier : un dossier est utilisé si l'un des fichiers
//! qu'il contient l'est.

#[cfg(target_os = "linux")]
mod proc;
#[cfg(target_os = "linux")]
use proc as os;
#[cfg(not(any(target_os = "linux", windows)))]
mod unsupported;
#[cfg(not(any(target_os = "linux", windows)))]
use unsupported as os;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as os;

use crate::safe_path::CanonicalPath;
use crate::units::ProcessName;

/// Ce que la vérification a établi pour un élément, à l'instant de la demande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenFileStatus {
    Free,
    /// Utilisé par ces logiciels (noms courts, sans doublon). La liste est vide quand le système dit « utilisé »
    /// sans dire par qui (Windows).
    InUse {
        programs: Vec<ProcessName>,
    },
    /// La détection a échoué ou n'est pas possible ici : l'appelant refuse par prudence (ADR 0020).
    Unavailable,
}

pub trait OpenFileInspector: Send + Sync {
    /// Interroge le système **au moment de l'appel** : le résultat n'est jamais gardé pour un autre élément.
    fn check(&self, item: &CanonicalPath) -> OpenFileStatus;
}

/// L'adapter du système en cours : instantané de `/proc` sous Linux, sonde d'ouverture avec accès `DELETE` sous
/// Windows, « impossible » ailleurs.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemOpenFiles;

impl SystemOpenFiles {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl OpenFileInspector for SystemOpenFiles {
    fn check(&self, item: &CanonicalPath) -> OpenFileStatus {
        os::check(item)
    }
}
