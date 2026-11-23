//! Exécution du plan hors du fil principal. Le cœur ne connaît pas Tauri (ADR 0015).

use std::fs;

use crate::service::{CleanupPorts, Reserved};
use crate::summary::{
    CleanupSummary, ItemView, ProgressEvent, SummaryStatus, failed_summary, summarize,
};
use breathly_disk::{
    CleanError, CleanupParts, CleanupReport, CleanupRun, Guardrails, JsonlJournal, run_cleanup,
};

pub type ProgressSink = Box<dyn FnMut(ProgressEvent) + Send>;

pub(crate) fn perform(
    ports: &CleanupPorts,
    reserved: &Reserved,
    sink: &mut ProgressSink,
) -> CleanupSummary {
    match run_plan(ports, reserved, sink) {
        Ok(report) => summarize(&report),
        Err(error) => failed_summary(
            bounded(reserved.plan.len()),
            view_of(error),
            SummaryStatus::Aborted,
        ),
    }
}

fn run_plan(
    ports: &CleanupPorts,
    reserved: &Reserved,
    sink: &mut ProgressSink,
) -> Result<CleanupReport, CleanError> {
    fs::create_dir_all(&ports.quarantine_dir).map_err(|_| CleanError::HistoryUnwritable)?;
    let mut journal = JsonlJournal::open(&ports.journal_path)?;
    let guardrails = Guardrails::new(ports.paths.as_ref(), ports.open_files.as_ref());
    let mut forward = |step: breathly_disk::ProgressMessage| {
        sink(ProgressEvent::Step {
            done: step.done,
            total: step.total,
            step_key: step.step_key,
        });
    };
    Ok(run_cleanup(CleanupRun {
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
    }))
}

fn view_of(error: CleanError) -> ItemView {
    ItemView {
        id: "setup".to_owned(),
        code: error.code(),
        translation_key: error.translation_key(),
    }
}

fn bounded(total: usize) -> u32 {
    u32::try_from(total).unwrap_or(u32::MAX)
}
