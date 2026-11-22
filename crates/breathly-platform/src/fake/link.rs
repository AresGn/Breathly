use std::io;
use std::path::Path;

/// Crée un lien de dossier (lien symbolique sous Linux, jonction sous Windows) pour les tests des garde-fous :
/// une jonction ne demande pas de droits administrateur (ADR 0019).
///
/// # Errors
/// Erreur d'entrée-sortie si le lien ne peut pas être créé.
pub fn link_directory(link: &Path, target: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }
    #[cfg(windows)]
    {
        junction(link, target)
    }
}

#[cfg(windows)]
#[allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "mklink /J crée une jonction sans droits administrateur : la bibliothèque standard n'en a pas (ADR 0019)"
)]
fn junction(link: &Path, target: &Path) -> io::Result<()> {
    let backslashed = |path: &Path| path.to_string_lossy().replace('/', "\\");
    let output = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(backslashed(link))
        .arg(backslashed(target))
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "mklink /J a échoué : {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}
