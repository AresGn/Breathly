//! Port `DiskInspector` : volumes, espace libre et total (design 0002 §5).

use std::path::PathBuf;

use crate::error::PlatformError;
use crate::units::{ByteSize, InvalidValue, check_part};

/// Le volume système porte l'alerte « moins de 2 Go libres » (US-020.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeRole {
    System,
    Other,
}

/// Espace total et disponible d'un volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeSpace {
    total: ByteSize,
    available: ByteSize,
}

impl VolumeSpace {
    /// # Errors
    /// `PartAboveTotal` si l'espace disponible dépasse le total.
    pub fn new(total: ByteSize, available: ByteSize) -> Result<Self, InvalidValue> {
        check_part(&available, &total)?;
        Ok(Self { total, available })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    mount_point: PathBuf,
    role: VolumeRole,
    space: VolumeSpace,
}

impl Volume {
    #[must_use]
    pub const fn new(mount_point: PathBuf, role: VolumeRole, space: VolumeSpace) -> Self {
        Self {
            mount_point,
            role,
            space,
        }
    }

    #[must_use]
    pub fn mount_point(&self) -> &std::path::Path {
        &self.mount_point
    }

    #[must_use]
    pub const fn role(&self) -> VolumeRole {
        self.role
    }

    #[must_use]
    pub const fn total(&self) -> ByteSize {
        self.space.total
    }

    #[must_use]
    pub const fn available(&self) -> ByteSize {
        self.space.available
    }
}

pub trait DiskInspector: Send + Sync {
    /// # Errors
    /// `MeasureUnavailable` si les volumes ne peuvent pas être lus.
    fn volumes(&self) -> Result<Vec<Volume>, PlatformError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_space_above_the_total_is_refused() {
        let (small, big) = (ByteSize::from_bytes(1), ByteSize::from_bytes(2));
        assert_eq!(
            VolumeSpace::new(small, big),
            Err(InvalidValue::PartAboveTotal)
        );
        let space = VolumeSpace::new(big, small).unwrap();
        let volume = Volume::new(PathBuf::from("/"), VolumeRole::System, space);
        assert_eq!((volume.total(), volume.available()), (big, small));
    }
}
