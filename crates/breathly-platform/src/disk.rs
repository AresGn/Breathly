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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    mount_point: PathBuf,
    role: VolumeRole,
    total: ByteSize,
    available: ByteSize,
}

impl Volume {
    /// # Errors
    /// `PartAboveTotal` si l'espace disponible dépasse le total.
    pub fn new(
        mount_point: PathBuf,
        role: VolumeRole,
        space: (ByteSize, ByteSize),
    ) -> Result<Self, InvalidValue> {
        let (total, available) = space;
        check_part(&available, &total)?;
        Ok(Self {
            mount_point,
            role,
            total,
            available,
        })
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
        self.total
    }

    #[must_use]
    pub const fn available(&self) -> ByteSize {
        self.available
    }
}

pub trait DiskInspector: Send + Sync {
    /// # Errors
    /// `MeasureUnavailable` si les volumes ne peuvent pas être lus.
    fn volumes(&self) -> Result<Vec<Volume>, PlatformError>;
}
