//! Plan figé par la simulation : le nettoyage n'accepte que lui, jamais un chemin venu de l'interface (SEC3).

use std::fmt;
use std::path::{Path, PathBuf};

use breathly_rules::Rule;

/// Élément choisi, tel que la simulation le fige. Le chemin reste dans le cœur.
pub struct SelectedItem {
    pub rule: Rule,
    pub roots: Vec<PathBuf>,
    pub path: PathBuf,
}

/// Identifiant d'un élément du plan. Ce n'est pas un chemin.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ItemId(String);

impl ItemId {
    pub(super) fn issued(number: u64) -> Self {
        Self(format!("i-{number}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ItemId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ItemId(..)")
    }
}

/// Un élément du plan, avec l'identifiant que la simulation lui a donné.
pub struct PlannedItem {
    id: ItemId,
    rule: Rule,
    roots: Vec<PathBuf>,
    path: PathBuf,
}

impl PlannedItem {
    pub(super) fn new(id: ItemId, item: SelectedItem) -> Self {
        Self {
            id,
            rule: item.rule,
            roots: item.roots,
            path: item.path,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &ItemId {
        &self.id
    }

    #[must_use]
    pub const fn rule(&self) -> &Rule {
        &self.rule
    }

    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Debug for PlannedItem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlannedItem")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}
