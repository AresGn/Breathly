//! Octets imprévisibles pour les jetons de confirmation (SEC3). Aucun secret n'est journalisé.

#[cfg(unix)]
use std::io::Read;

use crate::error::PlatformError;

/// Remplit `buffer` avec des octets fournis par le système.
///
/// # Errors
/// `EntropyUnavailable` si le système n'en fournit pas.
pub fn fill_random(buffer: &mut [u8]) -> Result<(), PlatformError> {
    read_entropy(buffer).map_err(|_| PlatformError::EntropyUnavailable)
}

#[cfg(unix)]
fn read_entropy(buffer: &mut [u8]) -> std::io::Result<()> {
    std::fs::File::open("/dev/urandom")?.read_exact(buffer)
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "RtlGenRandom (SystemFunction036) n'est pas dans la bibliothèque standard"
)]
fn read_entropy(buffer: &mut [u8]) -> std::io::Result<()> {
    for chunk in buffer.chunks_mut(u32::MAX as usize) {
        let length = u32::try_from(chunk.len()).unwrap_or(u32::MAX);
        // SAFETY: `chunk` est un tampon valide de `length` octets, écrit seulement par l'appel. `SystemFunction036`
        // (RtlGenRandom) est documenté pour remplir exactement ce tampon.
        let ok = unsafe { SystemFunction036(chunk.as_mut_ptr(), length) };
        if ok == 0 {
            return Err(std::io::Error::other("entropie indisponible"));
        }
    }
    Ok(())
}

#[cfg(windows)]
#[link(name = "advapi32")]
#[allow(
    unsafe_code,
    reason = "RtlGenRandom (SystemFunction036) n'est pas dans la bibliothèque standard"
)]
unsafe extern "system" {
    fn SystemFunction036(random_buffer: *mut u8, random_buffer_length: u32) -> u8;
}
