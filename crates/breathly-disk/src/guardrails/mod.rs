//! Garde-fous S5 et `VerifiedItem` (ticket #72, SEC2, ADR 0018 et 0019).
//!
//! Pour chaque élément, au moment d'agir : refus d'un niveau 🔴, élément hors de tout emplacement de la règle,
//! dossiers de l'utilisateur, profils de navigateur, conteneurs chiffrés, dossiers `.git`, puis ancrage sans suivre
//! de lien. Aucun réglage ne désactive un garde-fou : `Guardrails` n'en lit aucun. Les garde-fous « fichier ouvert »
//! (#73), liens symboliques (#74) et liste d'exclusion (#75) ont leur ticket.

mod browsers;
mod protected;
mod refusal;
mod verified;

use std::path::{Path, PathBuf};

use breathly_platform::{CanonicalPath, PathAnchor, StandardPaths};
use breathly_rules::Rule;

pub use refusal::Refusal;
pub use verified::VerifiedItem;

/// Un élément proposé par l'analyse : la règle qui le désigne, les emplacements de cette règle et son chemin.
///
/// `roots` sont les emplacements de la règle pour ce système, déjà résolus par l'analyse du disque (#56). Le chemin
/// n'est pas cru : il est canonicalisé et doit se trouver strictement sous l'un d'eux.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    pub rule: &'a Rule,
    pub roots: &'a [PathBuf],
    pub path: &'a Path,
}

/// Les garde-fous S5, revérifiés pour chaque élément.
pub struct Guardrails<'a> {
    paths: &'a dyn StandardPaths,
}

impl<'a> Guardrails<'a> {
    #[must_use]
    pub const fn new(paths: &'a dyn StandardPaths) -> Self {
        Self { paths }
    }

    /// Revérifie l'élément et renvoie son identité, ou le refus (un résultat typé : les autres éléments continuent).
    ///
    /// # Errors
    /// [`Refusal`] : niveau 🔴, hors règle, dossier protégé, profil de navigateur, conteneur chiffré, `.git`,
    /// chemin introuvable, `path_changed` ou `path_busy`.
    pub fn verify(&self, candidate: &Candidate<'_>) -> Result<VerifiedItem, Refusal> {
        let rule = candidate.rule;
        if !rule.risk().is_selectable() {
            return Err(Refusal::NotSelectable);
        }
        let item = CanonicalPath::resolve(candidate.path)?;
        let root = containing_root(candidate.roots, &item)?;
        protected::check(&item, self.paths)?;
        let anchor = PathAnchor::open(&root, &item)?;
        Ok(VerifiedItem::new(rule, item.display_path(), anchor))
    }
}

/// L'emplacement de la règle qui contient strictement l'élément : un emplacement lui-même n'est jamais un élément.
fn containing_root(roots: &[PathBuf], item: &CanonicalPath) -> Result<CanonicalPath, Refusal> {
    roots
        .iter()
        .filter_map(|root| CanonicalPath::resolve(root).ok())
        .find(|root| item.is_within(root) && !root.is_within(item))
        .ok_or(Refusal::OutsideRule)
}
