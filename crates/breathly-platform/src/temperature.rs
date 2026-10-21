//! Port `TemperatureReader` : température du processeur, « non disponible » sinon (D5, US-010.2).

use crate::units::Celsius;

/// Une lecture absente n'est pas une erreur : elle s'affiche « non disponible », jamais comme un palier (ADR 0022).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TemperatureReading {
    Available(Celsius),
    Unavailable,
}

pub trait TemperatureReader: Send + Sync {
    /// Une lecture ponctuelle ; `diagnose` en fait plusieurs pendant sa fenêtre (ADR 0022).
    fn cpu_temperature(&self) -> TemperatureReading;
}
