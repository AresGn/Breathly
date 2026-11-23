//! Une session de nettoyage à la fois, sur un fil nommé (ADR 0013).

use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread;

use breathly_disk::{CleanError, ConfirmationStore, PlannedItem, SelectedItem, StopFlag};
use breathly_platform::{
    Clock, OpenFileInspector, OsTrash, StandardPaths, SystemClock, SystemOpenFiles,
    SystemStandardPaths, TrashProvider, fill_random,
};

use crate::error::DesktopError;
use crate::summary::{CleanupSummary, ItemView, ProgressEvent, SummaryStatus, failed_summary};
use crate::worker::{self, ProgressSink};

pub struct CleanupPorts {
    pub paths: Arc<dyn StandardPaths>,
    pub open_files: Arc<dyn OpenFileInspector>,
    pub trash: Arc<dyn TrashProvider>,
    pub clock: Arc<dyn Clock>,
    pub journal_path: PathBuf,
    pub quarantine_dir: PathBuf,
}

#[derive(Clone)]
pub struct CleanupService {
    inner: Arc<Inner>,
}

struct Inner {
    ports: CleanupPorts,
    store: ConfirmationStore,
    session: Mutex<Session>,
    wake: Condvar,
}

struct Session {
    running: Option<Running>,
    finished: Option<Finished>,
}

struct Running {
    id: String,
    stop: Arc<StopFlag>,
}

struct Finished {
    id: String,
    summary: CleanupSummary,
}

pub(crate) struct Reserved {
    pub id: String,
    pub plan: Vec<PlannedItem>,
    pub stop: Arc<StopFlag>,
}

impl CleanupService {
    #[must_use]
    pub fn new(ports: CleanupPorts) -> Self {
        Self {
            inner: Arc::new(Inner {
                ports,
                store: ConfirmationStore::new(breathly_disk::CONFIRMATION_TTL),
                session: Mutex::new(Session {
                    running: None,
                    finished: None,
                }),
                wake: Condvar::new(),
            }),
        }
    }

    /// Composition réelle : corbeille du système, horloge, dossiers de l'application.
    #[must_use]
    pub fn system() -> Self {
        let paths: Arc<dyn StandardPaths> = Arc::new(SystemStandardPaths::new());
        let root = paths
            .state_dir()
            .unwrap_or_else(|_| PathBuf::from("/nonexistent-breathly-state"));
        Self::new(CleanupPorts {
            paths: Arc::clone(&paths),
            open_files: Arc::new(SystemOpenFiles::new()),
            trash: Arc::new(OsTrash::new()),
            clock: Arc::new(SystemClock::new()),
            journal_path: root.join("history.jsonl"),
            quarantine_dir: root.join("quarantine"),
        })
    }

    /// Point que `simulate_cleanup` appellera. Le jeton ne sort d'ici que vers l'interface.
    ///
    /// # Errors
    /// [`CleanError`] si la sélection est vide ou si l'aléa ou l'horloge manquent.
    pub fn issue_confirmation(&self, items: Vec<SelectedItem>) -> Result<String, CleanError> {
        let token = self
            .inner
            .store
            .issue(items, self.inner.ports.clock.now())?;
        Ok(token.as_str().to_owned())
    }

    /// Lance le plan du jeton. Rend la main tout de suite, avec l'identifiant de tâche.
    ///
    /// # Errors
    /// Jeton refusé, nettoyage déjà en cours, ou fil impossible à créer. Rien n'est modifié.
    pub fn start(&self, token: &str, sink: ProgressSink) -> Result<String, DesktopError> {
        let reserved = self.inner.reserve(token)?;
        let id = reserved.id.clone();
        let service = Arc::clone(&self.inner);
        let spawned = thread::Builder::new()
            .name("breathly-nettoyage".to_owned())
            .spawn(move || service.work(&reserved, sink));
        if spawned.is_err() {
            self.inner.clear(&id);
            return Err(DesktopError::ThreadFailed);
        }
        Ok(id)
    }

    /// Demande l'arrêt et attend le résumé (fait, non faits, deux chiffres).
    ///
    /// # Errors
    /// [`DesktopError::UnknownTask`] si l'identifiant n'est ni le nettoyage en cours ni le dernier terminé.
    pub fn stop(&self, task_id: &str) -> Result<CleanupSummary, DesktopError> {
        self.request_stop(task_id)?;
        self.wait_report(task_id)
    }

    /// Pose le drapeau d'arrêt sans attendre. Utile quand l'élément en cours est bloqué par le test.
    ///
    /// # Errors
    /// [`DesktopError::UnknownTask`].
    pub fn request_stop(&self, task_id: &str) -> Result<(), DesktopError> {
        let session = self.inner.lock();
        if let Some(running) = &session.running {
            if running.id != task_id {
                return Err(DesktopError::UnknownTask);
            }
            running.stop.request();
            return Ok(());
        }
        match &session.finished {
            Some(done) if done.id == task_id => Ok(()),
            _ => Err(DesktopError::UnknownTask),
        }
    }

    /// Attend le résumé du nettoyage `task_id`.
    ///
    /// # Errors
    /// [`DesktopError::UnknownTask`].
    pub fn wait_report(&self, task_id: &str) -> Result<CleanupSummary, DesktopError> {
        let mut session = self.inner.lock();
        loop {
            if session
                .running
                .as_ref()
                .is_some_and(|running| running.id == task_id)
            {
                session = self
                    .inner
                    .wake
                    .wait(session)
                    .unwrap_or_else(PoisonError::into_inner);
                continue;
            }
            return report_of(&session, task_id);
        }
    }
}

impl Inner {
    fn reserve(&self, token: &str) -> Result<Reserved, DesktopError> {
        let mut session = self.lock();
        if session.running.is_some() {
            return Err(DesktopError::AlreadyRunning);
        }
        let plan = self.plan_of(token)?;
        let id = task_id()?;
        let stop = Arc::new(StopFlag::new());
        session.running = Some(Running {
            id: id.clone(),
            stop: Arc::clone(&stop),
        });
        Ok(Reserved { id, plan, stop })
    }

    fn plan_of(&self, token: &str) -> Result<Vec<PlannedItem>, DesktopError> {
        let parsed = breathly_disk::ConfirmationToken::parse(token)?;
        Ok(self.store.consume(&parsed, self.ports.clock.now())?)
    }

    fn work(&self, reserved: &Reserved, mut sink: ProgressSink) {
        sink(ProgressEvent::Started);
        let summary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            worker::perform(&self.ports, reserved, &mut sink)
        }));
        let summary = summary.unwrap_or_else(|_| panicked(reserved.plan.len()));
        sink(crate::summary::terminal_event(&summary));
        self.finish(&reserved.id, summary);
    }

    fn finish(&self, id: &str, summary: CleanupSummary) {
        let mut session = self.lock();
        if session
            .running
            .as_ref()
            .is_some_and(|running| running.id == id)
        {
            session.running = None;
        }
        session.finished = Some(Finished {
            id: id.to_owned(),
            summary,
        });
        self.wake.notify_all();
    }

    fn clear(&self, id: &str) {
        self.finish(
            id,
            failed_summary(0, thread_failure(), SummaryStatus::Aborted),
        );
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Session> {
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn report_of(session: &Session, task_id: &str) -> Result<CleanupSummary, DesktopError> {
    match &session.finished {
        Some(done) if done.id == task_id => Ok(done.summary.clone()),
        _ => Err(DesktopError::UnknownTask),
    }
}

fn task_id() -> Result<String, DesktopError> {
    let mut bytes = [0_u8; 8];
    fill_random(&mut bytes).map_err(|_| CleanError::EntropyUnavailable)?;
    let mut text = String::with_capacity(16);
    for byte in bytes {
        text.push(hex(byte >> 4));
        text.push(hex(byte & 0x0f));
    }
    Ok(text)
}

fn hex(nibble: u8) -> char {
    char::from(b"0123456789abcdef"[usize::from(nibble)])
}

fn panicked(total: usize) -> CleanupSummary {
    failed_summary(bounded(total), panic_view(), SummaryStatus::Panicked)
}

fn panic_view() -> ItemView {
    ItemView {
        id: "task".to_owned(),
        code: "desktop.task_panicked",
        translation_key: "errors.desktop.task_panicked",
    }
}

fn thread_failure() -> ItemView {
    ItemView {
        id: "task".to_owned(),
        code: "desktop.thread_failed",
        translation_key: "errors.desktop.thread_failed",
    }
}

fn bounded(total: usize) -> u32 {
    u32::try_from(total).unwrap_or(u32::MAX)
}
