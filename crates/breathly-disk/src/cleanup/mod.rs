//! Nettoyage : jeton de confirmation, exécution par élément, arrêt demandé, résumé (ticket #88).

mod act;
mod error;
mod journal;
mod plan;
mod report;
mod run;
mod token;

pub use error::CleanError;
pub use journal::{
    CleanupJournal, Closed, Destination, EntryId, JournalOutcome, JsonlJournal, Pending,
};
pub use plan::{ItemId, PlannedItem, SelectedItem};
pub use report::{CleanupReport, CleanupStatus, ItemFailure, ProgressMessage, StopFlag};
pub use run::{CleanupParts, CleanupRun, run_cleanup, run_cleanup_into};
pub use token::{CONFIRMATION_TTL, ConfirmationStore, ConfirmationToken};
