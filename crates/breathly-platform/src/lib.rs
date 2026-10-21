//! Module `platform` : abstraction du système d'exploitation (processus, mémoire, disques, températures, corbeille, chemins standard). Seul crate qui peut contenir du code propre à un OS et, sous conditions, du `unsafe` (ADR 0011).
//!
//! Les ports (design 0002 §5) sont des traits synchrones (ADR 0013). Leurs faux, en mémoire, sont dans le module
//! `fake`, compilé avec la fonctionnalité `fake` que les autres crates activent en dépendance de développement.
//! Les adapters réels arrivent avec les tickets de chaque vague.

mod clock;
mod disk;
mod error;
mod process;
mod system;
mod temperature;
mod trash;
mod units;

#[cfg(any(test, feature = "fake"))]
pub mod fake;

pub use clock::Clock;
pub use disk::{DiskInspector, Volume, VolumeRole};
pub use error::{Measure, PlatformError};
pub use process::{
    Executable, ExecutableOrigin, MemoryUsage, Packaging, ProcessInspector, ProcessRole,
    ProcessSnapshot,
};
pub use system::{CpuTimes, MemoryStatus, SwapDevice, SwapKind, SwapStatus, SystemInspector};
pub use temperature::{TemperatureReader, TemperatureReading};
pub use trash::{TrashItem, TrashItemId, TrashProvider};
pub use units::{ByteSize, Celsius, InvalidValue, ProcessId, ProcessName};
