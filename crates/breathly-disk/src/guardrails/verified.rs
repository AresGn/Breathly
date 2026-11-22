use std::fmt;
use std::path::{Path, PathBuf};

use breathly_platform::PathAnchor;
use breathly_rules::{RiskLevel, Rule, RuleId};

/// Un élément dont les garde-fous ont été revérifiés au moment d'agir (SEC2, ADR 0018).
///
/// Seul le module des garde-fous le construit : le constructeur n'est pas visible du reste du crate, et aucun
/// `From<PathBuf>` n'existe. Il porte l'**identité** de l'élément (l'ancre : dossiers de la chaîne tenus ouverts
/// sans suivre de lien), pas un chemin revérifié : une suppression n'accepte que lui. Son `Debug` masque le
/// chemin (SEC9).
pub struct VerifiedItem {
    rule: RuleId,
    risk: RiskLevel,
    origin: PathBuf,
    anchor: PathAnchor,
}

impl VerifiedItem {
    pub(super) fn new(rule: &Rule, origin: PathBuf, anchor: PathAnchor) -> Self {
        Self {
            rule: rule.id().clone(),
            risk: rule.risk(),
            origin,
            anchor,
        }
    }

    #[must_use]
    pub const fn rule(&self) -> &RuleId {
        &self.rule
    }

    #[must_use]
    pub const fn risk(&self) -> RiskLevel {
        self.risk
    }

    /// Chemin d'origine, **pour l'historique et l'affichage seulement** (ADR 0018) : une action ne s'appuie jamais
    /// dessus, mais sur [`Self::anchor`].
    #[must_use]
    pub fn origin(&self) -> &Path {
        &self.origin
    }

    /// L'identité sur laquelle s'appuient les suppressions (tickets #81 et #82).
    #[must_use]
    pub const fn anchor(&self) -> &PathAnchor {
        &self.anchor
    }
}

impl fmt::Debug for VerifiedItem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedItem")
            .field("rule", &self.rule)
            .field("risk", &self.risk)
            .finish_non_exhaustive()
    }
}
