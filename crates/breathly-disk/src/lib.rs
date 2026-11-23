//! Modules `disk-scan` et `disk-clean` : analyse du disque, simulation, nettoyage, garde-fous et historique (design 0001 et 0002).
//!
//! Crate créé pour la V0.1 (ADR 0010). Garde-fous (`guardrails`) et nettoyage (`cleanup`).

mod cleanup;
mod guardrails;

pub use cleanup::{
    CONFIRMATION_TTL, CleanError, CleanupJournal, CleanupParts, CleanupReport, CleanupRun,
    CleanupStatus, Closed, ConfirmationStore, ConfirmationToken, Destination, EntryId, ItemFailure,
    ItemId, JournalOutcome, JsonlJournal, Pending, PlannedItem, ProgressMessage, SelectedItem,
    StopFlag, run_cleanup,
};
pub use guardrails::{Candidate, Guardrails, Refusal, VerifiedItem};
