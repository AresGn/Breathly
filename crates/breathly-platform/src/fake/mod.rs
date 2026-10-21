//! Faux des ports, pour les tests des autres crates (fonctionnalité `fake`, en dépendance de développement).
//!
//! Aucun faux ne lit ni n'écrit le vrai système : tout reste en mémoire (PRD §14).

mod trash;

use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, SystemTime};

use crate::clock::Clock;
use crate::disk::{DiskInspector, Volume};
use crate::error::PlatformError;
use crate::process::{ProcessInspector, ProcessSnapshot};
use crate::system::{CpuTimes, MemoryStatus, SwapStatus, SystemInspector};
use crate::temperature::{TemperatureReader, TemperatureReading};

pub use trash::MemoryTrash;

/// Valeurs rendues dans l'ordre, puis la dernière indéfiniment : deux relevés successifs d'une fenêtre (ADR 0022).
struct Sequence<T> {
    state: Mutex<(VecDeque<T>, T)>,
}

impl<T: Clone> Sequence<T> {
    fn new(first: T, rest: impl IntoIterator<Item = T>) -> Self {
        let mut pending: VecDeque<T> = rest.into_iter().collect();
        pending.push_front(first.clone());
        Self {
            state: Mutex::new((pending, first)),
        }
    }

    fn next(&self) -> T {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(value) = state.0.pop_front() {
            state.1 = value;
        }
        state.1.clone()
    }
}

/// Heure fixe, avançable à la main.
pub struct FixedClock {
    now: Mutex<SystemTime>,
}

impl FixedClock {
    #[must_use]
    pub const fn new(now: SystemTime) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, by: Duration) {
        let mut now = self.now.lock().unwrap_or_else(PoisonError::into_inner);
        *now += by;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        *self.now.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Température absente, fixe, ou une suite de lectures.
pub struct FakeTemperature(Sequence<TemperatureReading>);

impl FakeTemperature {
    #[must_use]
    pub fn fixed(reading: TemperatureReading) -> Self {
        Self(Sequence::new(reading, []))
    }

    #[must_use]
    pub fn readings(
        first: TemperatureReading,
        rest: impl IntoIterator<Item = TemperatureReading>,
    ) -> Self {
        Self(Sequence::new(first, rest))
    }
}

impl TemperatureReader for FakeTemperature {
    fn cpu_temperature(&self) -> TemperatureReading {
        self.0.next()
    }
}

type Snapshots = Result<Vec<ProcessSnapshot>, PlatformError>;

/// Liste fixe de processus, ou une suite de relevés.
pub struct FakeProcesses(Sequence<Snapshots>);

impl FakeProcesses {
    #[must_use]
    pub fn fixed(processes: Vec<ProcessSnapshot>) -> Self {
        Self(Sequence::new(Ok(processes), []))
    }

    #[must_use]
    pub fn snapshots(first: Snapshots, rest: impl IntoIterator<Item = Snapshots>) -> Self {
        Self(Sequence::new(first, rest))
    }
}

impl ProcessInspector for FakeProcesses {
    fn processes(&self) -> Snapshots {
        self.0.next()
    }
}

/// Mesures globales fixes ; les temps du processeur peuvent suivre une suite de relevés.
pub struct FakeSystem {
    memory: Result<MemoryStatus, PlatformError>,
    swap: Result<SwapStatus, PlatformError>,
    cpu: Sequence<Result<CpuTimes, PlatformError>>,
}

/// Réponses du faux `SystemInspector`.
pub struct SystemFacts {
    pub memory: Result<MemoryStatus, PlatformError>,
    pub swap: Result<SwapStatus, PlatformError>,
    pub cpu: Vec<Result<CpuTimes, PlatformError>>,
}

impl FakeSystem {
    /// Une liste `cpu` vide rend « mesure indisponible ».
    #[must_use]
    pub fn new(facts: SystemFacts) -> Self {
        let mut cpu = facts.cpu.into_iter();
        let unavailable = Err(PlatformError::MeasureUnavailable {
            measure: crate::error::Measure::Cpu,
        });
        let first = cpu.next().unwrap_or(unavailable);
        Self {
            memory: facts.memory,
            swap: facts.swap,
            cpu: Sequence::new(first, cpu),
        }
    }
}

impl SystemInspector for FakeSystem {
    fn memory(&self) -> Result<MemoryStatus, PlatformError> {
        self.memory.clone()
    }

    fn swap(&self) -> Result<SwapStatus, PlatformError> {
        self.swap.clone()
    }

    fn cpu_times(&self) -> Result<CpuTimes, PlatformError> {
        self.cpu.next()
    }
}

/// Volumes fixes.
pub struct FakeDisks(Result<Vec<Volume>, PlatformError>);

impl FakeDisks {
    #[must_use]
    pub const fn new(volumes: Result<Vec<Volume>, PlatformError>) -> Self {
        Self(volumes)
    }
}

impl DiskInspector for FakeDisks {
    fn volumes(&self) -> Result<Vec<Volume>, PlatformError> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Celsius;

    #[test]
    fn a_sequence_returns_its_values_in_order_then_repeats_the_last() {
        let hot = TemperatureReading::Available(Celsius::try_from(85.0).unwrap());
        let reader = FakeTemperature::readings(TemperatureReading::Unavailable, [hot]);
        assert_eq!(reader.cpu_temperature(), TemperatureReading::Unavailable);
        assert_eq!(reader.cpu_temperature(), hot);
        assert_eq!(reader.cpu_temperature(), hot);
    }

    #[test]
    fn the_fixed_clock_only_moves_when_advanced() {
        let clock = FixedClock::new(SystemTime::UNIX_EPOCH);
        assert_eq!(clock.now(), SystemTime::UNIX_EPOCH);
        clock.advance(Duration::from_secs(5));
        assert_eq!(clock.now(), SystemTime::UNIX_EPOCH + Duration::from_secs(5));
    }

    #[test]
    fn the_fake_system_without_cpu_readings_reports_them_unavailable() {
        let unavailable = |measure| PlatformError::MeasureUnavailable { measure };
        let system = FakeSystem::new(SystemFacts {
            memory: Err(unavailable(crate::error::Measure::Memory)),
            swap: Err(unavailable(crate::error::Measure::Swap)),
            cpu: Vec::new(),
        });
        assert_eq!(
            system.cpu_times(),
            Err(PlatformError::MeasureUnavailable {
                measure: crate::error::Measure::Cpu
            })
        );
    }
}
