//! Adapter réel de l'envoi à la corbeille (ADR 0005). Lister, restaurer et purger restent le ticket #71 :
//! `trash::delete` ne renvoie pas l'identifiant de la corbeille.

use std::path::Path;

use crate::error::PlatformError;
use crate::random::fill_random;
use crate::trash::{TrashItem, TrashItemId, TrashProvider};

/// Corbeille du système, pour l'envoi seulement.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsTrash;

impl OsTrash {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TrashProvider for OsTrash {
    fn send(&self, path: &Path) -> Result<TrashItemId, PlatformError> {
        trash::delete(path).map_err(|_| PlatformError::TrashUnavailable)?;
        Ok(TrashItemId::new(fresh_id()?))
    }

    fn list(&self) -> Result<Vec<TrashItem>, PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }

    fn restore(&self, _items: &[TrashItemId]) -> Result<(), PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }

    fn purge(&self, _items: &[TrashItemId]) -> Result<(), PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }
}

fn fresh_id() -> Result<String, PlatformError> {
    let mut bytes = [0_u8; 16];
    fill_random(&mut bytes)?;
    let mut id = String::from("os-trash/");
    for byte in bytes {
        id.push(hex_digit(byte >> 4));
        id.push(hex_digit(byte & 0x0f));
    }
    Ok(id)
}

fn hex_digit(value: u8) -> char {
    char::from(b"0123456789abcdef"[usize::from(value) % 16])
}
