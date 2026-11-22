use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use crate::open_file::{OpenFileInspector, OpenFileStatus};
use crate::safe_path::CanonicalPath;
use crate::units::ProcessName;

/// Fichiers « ouverts » choisis par le test, et une détection qui peut tomber en panne : rien ne touche au système.
///
/// Un dossier est utilisé si un fichier retenu se trouve dessous, comme avec les adapters réels. Les chemins
/// retenus sont canonicalisés à l'appel de [`Self::hold`] : ils existent donc.
#[derive(Default)]
pub struct FakeOpenFiles {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    held: Vec<(CanonicalPath, Vec<ProcessName>)>,
    is_unavailable: bool,
}

impl FakeOpenFiles {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Le fichier `path` devient ouvert par ces logiciels. Un chemin introuvable n'est pas retenu.
    pub fn hold(&self, path: impl Into<PathBuf>, programs: &[&str]) {
        let Ok(canonical) = CanonicalPath::resolve(&path.into()) else {
            return;
        };
        let names = programs
            .iter()
            .filter_map(|name| ProcessName::try_from((*name).to_owned()).ok())
            .collect();
        self.lock().held.push((canonical, names));
    }

    /// Plus aucun fichier n'est ouvert : le logiciel a été fermé.
    pub fn release_all(&self) {
        self.lock().held.clear();
    }

    /// La détection tombe en panne (ou revient) : `Unavailable` pour tout élément.
    pub fn set_unavailable(&self, is_unavailable: bool) {
        self.lock().is_unavailable = is_unavailable;
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl OpenFileInspector for FakeOpenFiles {
    fn check(&self, item: &CanonicalPath) -> OpenFileStatus {
        let state = self.lock();
        if state.is_unavailable {
            return OpenFileStatus::Unavailable;
        }
        let mut holders = state
            .held
            .iter()
            .filter(|(held, _)| held.is_within(item))
            .peekable();
        if holders.peek().is_none() {
            return OpenFileStatus::Free;
        }
        let mut programs: Vec<ProcessName> = holders
            .flat_map(|(_, names)| names.iter().cloned())
            .collect();
        programs.sort();
        programs.dedup();
        OpenFileStatus::InUse { programs }
    }
}
