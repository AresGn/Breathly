//! Port `SystemInspector` : mémoire, swap et processeur globaux (décision du 2026-10-18, ticket #30).

use std::num::NonZeroU16;
use std::time::Duration;

use crate::error::PlatformError;
use crate::units::{ByteSize, InvalidValue, check_part};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryStatus {
    total: ByteSize,
    available: ByteSize,
}

impl MemoryStatus {
    /// # Errors
    /// `PartAboveTotal` si la mémoire disponible dépasse le total.
    pub fn new(total: ByteSize, available: ByteSize) -> Result<Self, InvalidValue> {
        check_part(&available, &total)?;
        Ok(Self { total, available })
    }

    #[must_use]
    pub const fn total(&self) -> ByteSize {
        self.total
    }

    #[must_use]
    pub const fn available(&self) -> ByteSize {
        self.available
    }
}

/// Nature d'un périphérique de swap ; `Zram` dit si le swap compressé est actif (D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapKind {
    Zram,
    Partition,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapDevice {
    pub kind: SwapKind,
    pub size: ByteSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapStatus {
    total: ByteSize,
    used: ByteSize,
    devices: Vec<SwapDevice>,
}

impl SwapStatus {
    /// # Errors
    /// `PartAboveTotal` si le swap utilisé dépasse le total.
    pub fn new(
        total: ByteSize,
        used: ByteSize,
        devices: Vec<SwapDevice>,
    ) -> Result<Self, InvalidValue> {
        check_part(&used, &total)?;
        Ok(Self {
            total,
            used,
            devices,
        })
    }

    #[must_use]
    pub const fn total(&self) -> ByteSize {
        self.total
    }

    #[must_use]
    pub const fn used(&self) -> ByteSize {
        self.used
    }

    #[must_use]
    pub fn devices(&self) -> &[SwapDevice] {
        &self.devices
    }
}

/// Temps cumulés de tous les fils logiques ; la charge globale se calcule entre deux relevés (ADR 0022).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    busy: Duration,
    total: Duration,
    logical_cpus: NonZeroU16,
}

impl CpuTimes {
    /// # Errors
    /// `PartAboveTotal` si le temps occupé dépasse le temps total.
    pub fn new(
        busy: Duration,
        total: Duration,
        logical_cpus: NonZeroU16,
    ) -> Result<Self, InvalidValue> {
        check_part(&busy, &total)?;
        Ok(Self {
            busy,
            total,
            logical_cpus,
        })
    }

    #[must_use]
    pub const fn busy(&self) -> Duration {
        self.busy
    }

    #[must_use]
    pub const fn total(&self) -> Duration {
        self.total
    }

    #[must_use]
    pub const fn logical_cpus(&self) -> NonZeroU16 {
        self.logical_cpus
    }
}

pub trait SystemInspector: Send + Sync {
    /// # Errors
    /// `MeasureUnavailable` si la mémoire ne peut pas être lue.
    fn memory(&self) -> Result<MemoryStatus, PlatformError>;

    /// # Errors
    /// `MeasureUnavailable` si le swap ne peut pas être lu.
    fn swap(&self) -> Result<SwapStatus, PlatformError>;

    /// # Errors
    /// `MeasureUnavailable` si les temps du processeur ne peuvent pas être lus.
    fn cpu_times(&self) -> Result<CpuTimes, PlatformError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1 << 30;

    #[test]
    fn a_part_above_its_total_is_refused() {
        let (one, two) = (ByteSize::from_bytes(GIB), ByteSize::from_bytes(2 * GIB));
        assert!(MemoryStatus::new(two, one).is_ok());
        assert_eq!(
            MemoryStatus::new(one, two),
            Err(InvalidValue::PartAboveTotal)
        );
        assert_eq!(
            SwapStatus::new(one, two, Vec::new()),
            Err(InvalidValue::PartAboveTotal)
        );
        let cpus = NonZeroU16::MIN;
        let (short, long) = (Duration::from_secs(1), Duration::from_secs(2));
        assert_eq!(
            CpuTimes::new(long, short, cpus),
            Err(InvalidValue::PartAboveTotal)
        );
    }
}
