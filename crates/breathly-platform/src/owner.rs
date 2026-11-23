//! Droits limités au compte qui lance Breathly (SEC5). Sous Windows, le dossier du profil suffit.

use std::path::Path;

use crate::error::PlatformError;

/// Restreint `path` au propriétaire : `0700` pour un dossier, `0600` pour un fichier.
///
/// # Errors
/// `OwnerOnlyFailed` si les droits ne peuvent pas être posés. Sous Windows, l'appel réussit sans rien changer :
/// le fichier est déjà créé dans le profil de l'utilisateur.
pub fn limit_to_owner(path: &Path) -> Result<(), PlatformError> {
    apply(path).map_err(|_| PlatformError::OwnerOnlyFailed)
}

#[cfg(unix)]
fn apply(path: &Path) -> std::io::Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    let mode = if path.is_dir() { 0o700 } else { 0o600 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(windows)]
#[allow(
    clippy::unnecessary_wraps,
    reason = "même signature que la version Unix, qui peut échouer"
)]
fn apply(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
