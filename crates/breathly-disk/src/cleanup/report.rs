//! Résumé d'un nettoyage : deux chiffres, et les éléments supprimés, ignorés et en erreur (US-030.4, US-036).

use std::sync::atomic::{AtomicBool, Ordering};

use breathly_platform::ByteSize;

use super::plan::ItemId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupStatus {
    Completed,
    CompletedWithErrors,
    Stopped,
    Aborted,
}

pub(super) struct Freed {
    pub id: ItemId,
    pub size: ByteSize,
    pub to_trash: bool,
}

/// Un élément ignoré ou en erreur. Le code et la clé ne contiennent pas de chemin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFailure {
    pub id: ItemId,
    pub code: &'static str,
    pub translation_key: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupReport {
    pub status: CleanupStatus,
    pub trashed: ByteSize,
    pub freed_now: ByteSize,
    pub deleted: Vec<ItemId>,
    pub skipped: Vec<ItemFailure>,
    pub errors: Vec<ItemFailure>,
    pub done: u32,
    pub not_done: u32,
}

impl CleanupReport {
    #[must_use]
    pub fn new(total: u32) -> Self {
        Self {
            status: CleanupStatus::Completed,
            trashed: ByteSize::from_bytes(0),
            freed_now: ByteSize::from_bytes(0),
            deleted: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
            done: 0,
            not_done: total,
        }
    }

    pub(super) fn push_deleted(&mut self, freed: Freed) {
        if freed.to_trash {
            self.trashed = self.trashed.saturating_add(freed.size);
        } else {
            self.freed_now = self.freed_now.saturating_add(freed.size);
        }
        self.deleted.push(freed.id);
        self.count_one();
    }

    pub(super) fn push_skipped(&mut self, failure: ItemFailure) {
        self.skipped.push(failure);
        self.count_one();
    }

    pub(super) fn push_error(&mut self, failure: ItemFailure) {
        self.errors.push(failure);
        self.count_one();
    }

    /// Fixe l'état final et compte les éléments non faits.
    pub fn finish(&mut self, status: CleanupStatus) {
        self.status = status;
        self.not_done = self.not_done.saturating_sub(self.done);
    }

    fn count_one(&mut self) {
        self.done += 1;
    }
}

/// Demande d'arrêt consultée entre deux éléments (US-036.3).
#[derive(Debug, Default)]
pub struct StopFlag(AtomicBool);

impl StopFlag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Avancement d'un élément. `step_key` est une clé de traduction, jamais un chemin (SEC9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressMessage {
    pub done: u32,
    pub total: u32,
    pub step_key: &'static str,
}

pub(super) const STEP_KEY: &str = "cleanup.progress.item";
