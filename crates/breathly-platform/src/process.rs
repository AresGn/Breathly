//! Port `ProcessInspector` : les processus et les faits dont `diagnose` a besoin (ADR 0021, ADR 0022).

use std::path::PathBuf;
use std::time::Duration;

use crate::error::PlatformError;
use crate::units::{ByteSize, ProcessId, ProcessName};

/// Origine d'un exécutable lisible (ADR 0021).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutableOrigin {
    /// Le fichier et ses dossiers parents appartiennent au compte système et ne sont modifiables par personne d'autre.
    Protected,
    /// Modifiable par un utilisateur standard.
    Writable,
    /// Dans un dossier temporaire, hors montage `AppImage`.
    Temporary,
    /// Dans un montage `AppImage` (`.mount_*` sous le dossier temporaire).
    AppImageMount,
    /// Effacé après le lancement, mais remplacé par un fichier protégé ou fourni par un paquet (mise à jour).
    DeletedReplaced,
    /// Effacé après le lancement, sans remplaçant.
    DeletedWithoutReplacement,
    /// Fichier en mémoire (`memfd`).
    InMemory,
}

/// Ce que le gestionnaire de paquets dit de l'exécutable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packaging {
    ProvidedByPackage,
    NotPackaged,
    /// Aucune base de paquets lisible (et toujours sous Windows).
    Unknown,
}

/// L'exécutable d'un processus, tel que Breathly peut le lire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Executable {
    /// Autre compte, droits insuffisants, ou thread du noyau.
    Unreadable,
    Readable {
        path: PathBuf,
        origin: ExecutableOrigin,
        packaging: Packaging,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRole {
    KernelThread,
    SystemService,
    Regular,
}

/// Mémoire d'un processus : PSS quand elle est lisible, RSS sinon (ADR 0021).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryUsage {
    Proportional(ByteSize),
    Resident(ByteSize),
}

impl MemoryUsage {
    #[must_use]
    pub const fn size(self) -> ByteSize {
        match self {
            Self::Proportional(size) | Self::Resident(size) => size,
        }
    }
}

/// Un processus à un instant donné.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSnapshot {
    pub pid: ProcessId,
    pub name: ProcessName,
    pub executable: Executable,
    pub role: ProcessRole,
    pub memory: MemoryUsage,
    /// Temps de processeur cumulé depuis le lancement ; la charge se calcule entre deux relevés (ADR 0022).
    pub cpu_time: Duration,
}

pub trait ProcessInspector: Send + Sync {
    /// Relève tous les processus visibles.
    ///
    /// # Errors
    /// `MeasureUnavailable` si la liste des processus ne peut pas être lue du tout.
    fn processes(&self) -> Result<Vec<ProcessSnapshot>, PlatformError>;
}
