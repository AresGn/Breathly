//! Modules `disk-scan` et `disk-clean` : analyse du disque, simulation, nettoyage, garde-fous et historique (design 0001 et 0002).
//!
//! Crate créé pour la V0.1 (ADR 0010). Pour l'instant : les garde-fous (`guardrails`, ticket #72) et la simulation avec
//! son jeton (`cleanup`, ticket #76) ; le reste arrive avec les tickets de la version.

mod cleanup;
mod guardrails;

pub use cleanup::{
    CONFIRMATION_TTL, CleanError, ConfirmationStore, ConfirmationToken, Destination, Foreseen,
    Issued, ItemId, PlannedItem, Reason, SelectedItem, SimulatedItem, Simulation, SimulationReport,
    SimulationRequest, foresee, simulate,
};
pub use guardrails::{Candidate, Guardrails, Refusal, VerifiedItem};
