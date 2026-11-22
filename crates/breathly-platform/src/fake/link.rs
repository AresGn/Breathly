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
    reason = "mklink /J crée une jonction sans droits administrateur : la bibliothèque standard n'en a pas (ADR 0019)"
)]
fn junction(link: &Path, target: &Path) -> io::Result<()> {
    let status = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(link)
        .arg(target)
        .output()?
        .status;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("mklink /J a échoué"))
    }
}
