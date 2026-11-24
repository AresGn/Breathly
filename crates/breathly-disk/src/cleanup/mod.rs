//! Nettoyage : simulation et jeton de confirmation à usage unique (ticket #76). L'exécuteur, les destinations et
//! l'arrêt demandé arrivent avec leurs tickets (#80, #81, #82, #83).

mod destination;
mod error;
mod plan;
mod simulate;
mod token;

pub use destination::Destination;
pub use error::CleanError;
pub use plan::{ItemId, PlannedItem, SelectedItem};
pub use simulate::{
    Foreseen, Reason, SimulatedItem, Simulation, SimulationReport, SimulationRequest, foresee,
    simulate,
};
pub use token::{CONFIRMATION_TTL, ConfirmationStore, ConfirmationToken, Issued};
