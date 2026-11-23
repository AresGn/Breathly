//! Banc du test de nettoyage : dossier temporaire, journal et corbeille de test.

use std::fs;
use std::path::{Path, PathBuf};

use breathly_disk::{
    CONFIRMATION_TTL, CleanError, CleanupJournal, CleanupParts, CleanupReport, CleanupRun, Closed,
    ConfirmationStore, EntryId, Guardrails, JournalOutcome, JsonlJournal, Pending, PlannedItem,
    ProgressMessage, SelectedItem, StopFlag, run_cleanup,
};
use breathly_platform::TrashProvider;
use breathly_platform::fake::{FixedClock, MemoryTrash};

use super::NOW;
use super::adapters::BridgedTrash;
use super::common::{Fixture, rule};

pub(crate) struct World {
    fixture: Fixture,
    pub(crate) trash: MemoryTrash,
    clock: FixedClock,
}

impl World {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            fixture: Fixture::new()?,
            trash: MemoryTrash::new(),
            clock: FixedClock::new(NOW),
        })
    }

    pub(crate) fn sandbox(&self) -> PathBuf {
        self.fixture.sandbox.path().to_path_buf()
    }

    pub(crate) fn cache_file(&self, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.fixture.file(&format!(".cache/demo/{name}"))
    }

    pub(crate) fn item(
        &self,
        risk: &str,
        path: &Path,
    ) -> Result<SelectedItem, Box<dyn std::error::Error>> {
        Ok(SelectedItem {
            rule: rule(risk)?,
            roots: vec![self.fixture.home().join(".cache/demo")],
            path: path.to_path_buf(),
        })
    }

    pub(crate) fn bridge(&self) -> BridgedTrash<'_> {
        BridgedTrash::new(&self.trash)
    }

    pub(crate) fn clean(
        &self,
        attempt: &Attempt<'_>,
    ) -> Result<CleanupReport, Box<dyn std::error::Error>> {
        let mut journal = JsonlJournal::open(&self.sandbox().join("history.jsonl"))?;
        let mut progress = |_| {};
        let mut drive = Drive {
            plan: attempt.plan,
            stop: attempt.stop,
            trash: attempt.trash,
            journal: &mut journal,
            progress: &mut progress,
        };
        Ok(self.drive(&mut drive)?)
    }

    pub(crate) fn clean_progress(
        &self,
        heard: &mut Heard<'_>,
    ) -> Result<CleanupReport, Box<dyn std::error::Error>> {
        let stop = StopFlag::new();
        let mut journal = JsonlJournal::open(&self.sandbox().join("history.jsonl"))?;
        let mut drive = Drive {
            plan: heard.plan,
            stop: &stop,
            trash: heard.trash,
            journal: &mut journal,
            progress: &mut *heard.progress,
        };
        Ok(self.drive(&mut drive)?)
    }

    pub(crate) fn clean_journal(
        &self,
        noted: &mut Noted<'_>,
    ) -> Result<CleanupReport, Box<dyn std::error::Error>> {
        let stop = StopFlag::new();
        let mut progress = |_| {};
        let mut drive = Drive {
            plan: noted.plan,
            stop: &stop,
            trash: noted.trash,
            journal: &mut *noted.journal,
            progress: &mut progress,
        };
        Ok(self.drive(&mut drive)?)
    }

    fn drive(&self, drive: &mut Drive<'_>) -> Result<CleanupReport, CleanError> {
        let quarantine = self.sandbox().join("quarantine");
        fs::create_dir_all(&quarantine).map_err(|_| CleanError::HistoryUnwritable)?;
        let guardrails = Guardrails::new(&self.fixture.paths, &self.fixture.open_files);
        Ok(run_cleanup(CleanupRun {
            plan: drive.plan,
            parts: CleanupParts {
                guardrails,
                trash: drive.trash,
                journal: &mut *drive.journal,
                quarantine: &quarantine,
                clock: &self.clock,
            },
            stop: drive.stop,
            progress: &mut *drive.progress,
        }))
    }
}

pub(crate) fn plan(
    items: Vec<SelectedItem>,
) -> Result<Vec<PlannedItem>, Box<dyn std::error::Error>> {
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    Ok(store.consume(&store.issue(items, NOW)?, NOW)?)
}

pub(crate) struct Attempt<'a> {
    pub plan: &'a [PlannedItem],
    pub stop: &'a StopFlag,
    pub trash: &'a dyn TrashProvider,
}

pub(crate) struct Heard<'a> {
    pub plan: &'a [PlannedItem],
    pub trash: &'a dyn TrashProvider,
    pub progress: &'a mut dyn FnMut(ProgressMessage),
}

pub(crate) struct Noted<'a> {
    pub plan: &'a [PlannedItem],
    pub trash: &'a dyn TrashProvider,
    pub journal: &'a mut dyn CleanupJournal,
}

pub(crate) struct Drive<'a> {
    plan: &'a [PlannedItem],
    stop: &'a StopFlag,
    trash: &'a dyn TrashProvider,
    journal: &'a mut dyn CleanupJournal,
    progress: &'a mut dyn FnMut(ProgressMessage),
}

pub(crate) struct BrokenJournal;

impl CleanupJournal for BrokenJournal {
    fn record_pending(&mut self, _pending: Pending<'_>) -> Result<EntryId, CleanError> {
        Err(CleanError::HistoryUnwritable)
    }

    fn record_outcome(&mut self, _id: EntryId, _closed: Closed) -> Result<(), CleanError> {
        Err(CleanError::HistoryUnwritable)
    }

    fn record_note(
        &mut self,
        _pending: Pending<'_>,
        _outcome: JournalOutcome,
    ) -> Result<(), CleanError> {
        Err(CleanError::HistoryUnwritable)
    }
}

pub(crate) struct WatchingJournal {
    pub inner: JsonlJournal,
    pub saw_file: bool,
}

impl CleanupJournal for WatchingJournal {
    fn record_pending(&mut self, pending: Pending<'_>) -> Result<EntryId, CleanError> {
        self.saw_file = pending.path.exists();
        self.inner.record_pending(pending)
    }

    fn record_outcome(&mut self, id: EntryId, closed: Closed) -> Result<(), CleanError> {
        self.inner.record_outcome(id, closed)
    }

    fn record_note(
        &mut self,
        pending: Pending<'_>,
        outcome: JournalOutcome,
    ) -> Result<(), CleanError> {
        self.inner.record_note(pending, outcome)
    }
}
