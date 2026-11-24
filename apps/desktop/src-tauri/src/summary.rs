//! Résumé et progression vus par l'interface : aucun chemin (SEC3, SEC9).

use serde::Serialize;

use breathly_disk::{CleanupReport, CleanupStatus, ItemFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SummaryStatus {
    Completed,
    CompletedWithErrors,
    Stopped,
    Aborted,
    Panicked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: String,
    pub code: &'static str,
    pub translation_key: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupSummary {
    pub status: SummaryStatus,
    pub trashed_bytes: u64,
    pub freed_now_bytes: u64,
    pub done: u32,
    pub not_done: u32,
    pub deleted: Vec<String>,
    pub skipped: Vec<ItemView>,
    pub errors: Vec<ItemView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProgressEvent {
    Started,
    Step {
        done: u32,
        total: u32,
        step: &'static str,
    },
    Finished,
    Cancelled,
    Failed {
        error: ErrorView,
    },
}

/// L'erreur typée du contrat : un code stable et une clé de traduction (ADR 0015).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorView {
    pub code: &'static str,
    pub translation_key: &'static str,
}

impl ProgressEvent {
    pub(crate) fn is_step(&self) -> bool {
        matches!(self, Self::Step { .. })
    }
}

pub(crate) fn summarize(report: &CleanupReport) -> CleanupSummary {
    CleanupSummary {
        status: status_of(report.status),
        trashed_bytes: report.trashed.bytes(),
        freed_now_bytes: report.freed_now.bytes(),
        done: report.done,
        not_done: report.not_done,
        deleted: report
            .deleted
            .iter()
            .map(|id| id.as_str().to_owned())
            .collect(),
        skipped: report.skipped.iter().map(view_of).collect(),
        errors: report.errors.iter().map(view_of).collect(),
    }
}

/// Ce qui était déjà fait reste vrai : le résumé part du rapport, puis ajoute la cause de l'échec.
pub(crate) fn aborted_summary(
    report: &CleanupReport,
    status: SummaryStatus,
    view: ItemView,
) -> CleanupSummary {
    let mut summary = summarize(report);
    summary.status = status;
    summary.errors.push(view);
    summary
}

pub(crate) fn failed_summary(total: u32, view: ItemView, status: SummaryStatus) -> CleanupSummary {
    CleanupSummary {
        status,
        trashed_bytes: 0,
        freed_now_bytes: 0,
        done: 0,
        not_done: total,
        deleted: Vec::new(),
        skipped: Vec::new(),
        errors: vec![view],
    }
}

pub(crate) fn terminal_event(summary: &CleanupSummary) -> ProgressEvent {
    match summary.status {
        SummaryStatus::Stopped => ProgressEvent::Cancelled,
        SummaryStatus::Aborted | SummaryStatus::Panicked => failed_event(summary),
        SummaryStatus::Completed | SummaryStatus::CompletedWithErrors => ProgressEvent::Finished,
    }
}

fn failed_event(summary: &CleanupSummary) -> ProgressEvent {
    let error = summary.errors.first().map_or(
        ErrorView {
            code: "desktop.task_panicked",
            translation_key: "errors.desktop.task_panicked",
        },
        |failure| ErrorView {
            code: failure.code,
            translation_key: failure.translation_key,
        },
    );
    ProgressEvent::Failed { error }
}

fn status_of(status: CleanupStatus) -> SummaryStatus {
    match status {
        CleanupStatus::Completed => SummaryStatus::Completed,
        CleanupStatus::CompletedWithErrors => SummaryStatus::CompletedWithErrors,
        CleanupStatus::Stopped => SummaryStatus::Stopped,
        CleanupStatus::Aborted => SummaryStatus::Aborted,
    }
}

fn view_of(failure: &ItemFailure) -> ItemView {
    ItemView {
        id: failure.id.as_str().to_owned(),
        code: failure.code,
        translation_key: failure.translation_key,
    }
}
