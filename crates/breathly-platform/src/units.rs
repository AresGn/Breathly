//! Types de valeur partagés par les traits : tailles, identifiants, températures.

use thiserror::Error;

/// Valeur refusée à la construction d'un type de mesure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidValue {
    #[error("température hors de l'intervalle plausible")]
    TemperatureOutOfRange,
    #[error("une part dépasse le total")]
    PartAboveTotal,
    #[error("nom vide")]
    EmptyName,
}

/// Taille en octets (RAM, disque, fichier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ByteSize(u64);

impl ByteSize {
    #[must_use]
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
}

/// Identifiant d'un processus attribué par l'OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u32);

impl ProcessId {
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Nom court d'un processus tel que l'OS le donne (mode avancé seulement).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessName(String);

impl ProcessName {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProcessName {
    type Error = InvalidValue;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(InvalidValue::EmptyName);
        }
        Ok(Self(raw))
    }
}

/// Température en degrés Celsius.
///
/// Une valeur hors de [-50, 150] °C vient d'un capteur défaillant : l'adapter la rapporte comme « non disponible »
/// plutôt que de l'afficher (US-010.2).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Celsius(f32);

impl Celsius {
    const MIN: f32 = -50.0;
    const MAX: f32 = 150.0;

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for Celsius {
    type Error = InvalidValue;

    fn try_from(raw: f32) -> Result<Self, Self::Error> {
        if raw.is_finite() && (Self::MIN..=Self::MAX).contains(&raw) {
            Ok(Self(raw))
        } else {
            Err(InvalidValue::TemperatureOutOfRange)
        }
    }
}

/// Vérifie qu'une part (disponible, utilisé, occupé) ne dépasse pas son total.
pub(crate) fn check_part<T: PartialOrd>(part: &T, total: &T) -> Result<(), InvalidValue> {
    if part > total {
        return Err(InvalidValue::PartAboveTotal);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_accepts_plausible_values_and_refuses_the_rest() {
        assert_eq!(Celsius::try_from(81.5).map(Celsius::get), Ok(81.5));
        assert_eq!(Celsius::try_from(150.0).map(Celsius::get), Ok(150.0));
        assert_eq!(
            Celsius::try_from(150.1),
            Err(InvalidValue::TemperatureOutOfRange)
        );
        assert_eq!(
            Celsius::try_from(-50.1),
            Err(InvalidValue::TemperatureOutOfRange)
        );
        assert_eq!(
            Celsius::try_from(f32::NAN),
            Err(InvalidValue::TemperatureOutOfRange)
        );
    }

    #[test]
    fn process_name_refuses_an_empty_name() {
        assert_eq!(
            ProcessName::try_from(String::new()),
            Err(InvalidValue::EmptyName)
        );
        assert_eq!(
            ProcessName::try_from("code".to_owned()).map(|n| n.as_str().to_owned()),
            Ok("code".to_owned())
        );
    }

    #[test]
    fn byte_size_addition_saturates() {
        let max = ByteSize::from_bytes(u64::MAX);
        assert_eq!(max.saturating_add(ByteSize::from_bytes(1)), max);
    }
}
