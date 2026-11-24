//! Exécution du plan hors du fil principal. Le cœur ne connaît pas Tauri (ADR 0015).

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::Sender;

use crate::service::{CleanupPorts, Reserved};
use crate::summary::{
    CleanupSummary, ItemView, ProgressEvent, SummaryStatus, aborted_summary, summarize,
};
use breathly_disk::{
    CleanError, CleanupParts, CleanupReport, CleanupRun, CleanupStatus, Guardrails, JsonlJournal,
    run_cleanup_into,
};

pub type ProgressSink = Box<dyn FnMut(ProgressEvent) + Send>;

/// Rend le résumé même si le nettoyage panique : le rapport appartient à cette fonction, il garde ce qui était fait.
pub(crate) fn perform(
    ports: &CleanupPorts,
    reserved: &Reserved,
    progress: &Sender<ProgressEvent>,
) -> CleanupSummary {
    let mut report = CleanupReport::new(bounded(reserved.plan.len()));
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        run_plan(
            &Job {
                ports,
                reserved,
                progress,
            },
            &mut report,
        )
    }));
    match outcome {
        Ok(Ok(())) => summarize(&report),
        Ok(Err(error)) => aborted(&mut report, SummaryStatus::Aborted, view_of(error)),
        Err(_) => aborted(&mut report, SummaryStatus::Panicked, panic_view()),
    }
}

fn aborted(report: &mut CleanupReport, status: SummaryStatus, view: ItemView) -> CleanupSummary {
    report.finish(CleanupStatus::Aborted);
    aborted_summary(report, status, view)
}

struct Job<'a> {
    ports: &'a CleanupPorts,
    reserved: &'a Reserved,
    progress: &'a Sender<ProgressEvent>,
}

fn run_plan(job: &Job<'_>, report: &mut CleanupReport) -> Result<(), CleanError> {
    let Job {
        ports,
        reserved,
        progress,
    } = job;
    fs::create_dir_all(&ports.quarantine_dir).map_err(|_| CleanError::QuarantineUnavailable)?;
    let mut journal = JsonlJournal::open(&ports.journal_path)?;
    let guardrails = Guardrails::new(ports.paths.as_ref(), ports.open_files.as_ref());
    let mut forward = |message: breathly_disk::ProgressMessage| {
        emit(
            progress,
            ProgressEvent::Step {
                done: message.done,
                total: message.total,
                step: message.step_key,
            },
        );
    };
    run_cleanup_into(
        CleanupRun {
            plan: &reserved.plan,
            parts: CleanupParts {
                guardrails,
                trash: ports.trash.as_ref(),
                journal: &mut journal,
                quarantine: &ports.quarantine_dir,
                clock: ports.clock.as_ref(),
            },
            stop: &reserved.stop,
            progress: &mut forward,
        },
        report,
    );
    Ok(())
}

/// Un message perdu (relais arrêté) n'interrompt pas le nettoyage : il est journalisé avec un code (ADR 0015, SEC9).
pub(crate) fn emit(progress: &Sender<ProgressEvent>, event: ProgressEvent) {
    if progress.send(event).is_err() {
        tracing::warn!(code = "desktop.progress_lost", "progression non livrée");
    }
}

fn view_of(error: CleanError) -> ItemView {
    ItemView {
        id: "setup".to_owned(),
        code: error.code(),
        translation_key: error.translation_key(),
    }
}

fn panic_view() -> ItemView {
    ItemView {
        id: "task".to_owned(),
        code: "desktop.task_panicked",
        translation_key: "errors.desktop.task_panicked",
    }
}

fn bounded(total: usize) -> u32 {
    u32::try_from(total).unwrap_or(u32::MAX)
}
