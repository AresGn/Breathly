//! Où va un élément : la corbeille, ou la suppression définitive (ADR 0005, invariant 2).

use breathly_rules::RiskLevel;

/// Seul un cache 🟢 se supprime définitivement ; un fichier de l'utilisateur 🟡 va à la corbeille ; un 🔴 n'a aucune
/// destination. Simulation et nettoyage lisent cette seule règle : ils ne peuvent pas diverger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Trash,
    Permanent,
}

impl Destination {
    #[must_use]
    pub const fn of(risk: RiskLevel) -> Option<Self> {
        match risk {
            RiskLevel::Safe => Some(Self::Permanent),
            RiskLevel::Review => Some(Self::Trash),
            RiskLevel::Personal => None,
        }
    }
}
