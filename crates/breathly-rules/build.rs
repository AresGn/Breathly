//! Intègre les fichiers `rules/*.toml` du dépôt au binaire (ADR 0004) : ce script les liste à la compilation et
//! écrit un tableau `include_str!`. À l'exécution, aucun dossier de règles n'est lu.
//!
//! Le dossier `rules/` est celui du workspace (`../../rules`) : un build du crate seul, hors du dépôt, n'est pas
//! prévu et échoue avec un message qui le dit.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or_else(missing_variable)?);
    let rules_dir = manifest.join("..").join("..").join("rules");
    if !rules_dir.is_dir() {
        return Err(invalid(format!(
            "dossier rules/ introuvable à {}",
            rules_dir.display()
        )));
    }
    println!("cargo:rerun-if-changed={}", rules_dir.display());
    let files = rule_files(&rules_dir)?;
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or_else(missing_variable)?);
    fs::write(out.join("embedded_rules.rs"), generated(&files)?)
}

/// Fichiers `.toml` directement dans `rules/`, triés pour un binaire reproductible. Une entrée qui ressemble à
/// une règle sans en être une (dossier `x.toml`, extension `.TOML`) arrête la compilation au lieu d'être ignorée.
fn rule_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !extension.eq_ignore_ascii_case("toml") {
            continue;
        }
        if extension != "toml" || !entry.file_type()?.is_file() {
            return Err(invalid(format!(
                "{} : une règle est un fichier dont l'extension est exactement « .toml »",
                path.display()
            )));
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

fn generated(files: &[PathBuf]) -> io::Result<String> {
    let mut code = String::from("&[\n");
    for file in files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid("nom de fichier de règles non UTF-8".to_owned()))?;
        writeln!(
            code,
            "    ({name:?}, include_str!({:?})),",
            file.display().to_string()
        )
        .map_err(io::Error::other)?;
    }
    code.push(']');
    Ok(code)
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn missing_variable() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "variable de Cargo absente")
}
