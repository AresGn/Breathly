//! Déroulement du nettoyage : un élément à la fois, arrêt entre deux, erreur sans arrêt global.

use std::path::Path;

use breathly_platform::{Clock, TrashProvider};
use breathly_rules::RiskLevel;

use super::act::{self, ActionIo, ActionKind, Perform};
use super::journal::{CleanupJournal, Destination, JournalOutcome};
use super::plan::PlannedItem;
use super::report::{
    CleanupReport, CleanupStatus, ItemFailure, ProgressMessage, STEP_KEY, StopFlag,
};
use crate::{Candidate, Guardrails, Refusal, VerifiedItem};

pub struct CleanupParts<'a> {
    pub guardrails: Guardrails<'a>,
    pub trash: &'a dyn TrashProvider,
    pub journal: &'a mut dyn CleanupJournal,
    pub quarantine: &'a Path,
    pub clock: &'a dyn Clock,
}

pub struct CleanupRun<'a> {
    pub plan: &'a [PlannedItem],
    pub parts: CleanupParts<'a>,
    pub stop: &'a StopFlag,
    pub progress: &'a mut dyn FnMut(ProgressMessage),
}

/// Exécute le plan. `stop` est consulté avant chaque élément.
#[must_use]
pub fn run_cleanup(run: CleanupRun<'_>) -> CleanupReport {
    let total = u32::try_from(run.plan.len()).unwrap_or(u32::MAX);
    let mut report = CleanupReport::new(total);
    run_cleanup_into(run, &mut report);
    report
}

/// Comme [`run_cleanup`], mais le résumé appartient à l'appelant : si l'exécution panique, il garde ce qui était
/// déjà fait (les deux chiffres restent vrais).
pub fn run_cleanup_into(mut run: CleanupRun<'_>, report: &mut CleanupReport) {
    let total = u32::try_from(run.plan.len()).unwrap_or(u32::MAX);
    for item in run.plan {
        if run.stop.is_requested() {
            report.finish(CleanupStatus::Stopped);
            return;
        }
        if !apply_item(item, &mut run.parts, report) {
            report.finish(CleanupStatus::Aborted);
            return;
        }
        (run.progress)(ProgressMessage {
            done: report.done,
            total,
            step_key: STEP_KEY,
        });
    }
    report.finish(closing_status(report, run.stop));
}

/// Un arrêt demandé pendant le dernier élément finit en `Stopped` (design 0001 §2 : Stopping, `item_done`, Stopped).
fn closing_status(report: &CleanupReport, stop: &StopFlag) -> CleanupStatus {
    if stop.is_requested() {
        CleanupStatus::Stopped
    } else {
        final_status(report)
    }
}

fn apply_item(
    item: &PlannedItem,
    parts: &mut CleanupParts<'_>,
    report: &mut CleanupReport,
) -> bool {
    let verified = match parts.guardrails.verify(&candidate(item)) {
        Ok(verified) => verified,
        Err(refusal) => {
            return note_refusal(
                parts,
                report,
                &Skip {
                    item,
                    refusal: &refusal,
                },
            );
        }
    };
    record_action(
        parts,
        report,
        &Act {
            item,
            verified: &verified,
        },
    )
}

struct Act<'a> {
    item: &'a PlannedItem,
    verified: &'a VerifiedItem,
}

fn record_action(parts: &mut CleanupParts<'_>, report: &mut CleanupReport, step: &Act<'_>) -> bool {
    let mut io = ActionIo {
        trash: parts.trash,
        journal: parts.journal,
        quarantine: parts.quarantine,
        now: parts.clock.now(),
    };
    match act::perform(step.verified, &mut io) {
        Perform::Continue(done) => {
            push(report, &record_of(step.item, &done));
            true
        }
        Perform::Halt(Some(done)) => {
            push(report, &record_of(step.item, &done));
            false
        }
        Perform::Halt(None) => false,
    }
}

fn record_of<'a>(item: &'a PlannedItem, action: &act::Action) -> Record<'a> {
    Record {
        item,
        kind: action.kind,
        size: action.size,
        failure: action.failure,
    }
}

struct Skip<'a> {
    item: &'a PlannedItem,
    refusal: &'a Refusal,
}

fn note_refusal(parts: &mut CleanupParts<'_>, report: &mut CleanupReport, skip: &Skip<'_>) -> bool {
    let pending = super::journal::Pending {
        at: parts.clock.now(),
        rule_id: skip.item.rule().id().as_str(),
        path: skip.item.path(),
        size: breathly_platform::ByteSize::from_bytes(0),
        destination: intended(skip.item.rule().risk()),
    };
    if parts
        .journal
        .record_note(pending, JournalOutcome::Skipped)
        .is_err()
    {
        return false;
    }
    let detail = failure(
        skip.item,
        skip.refusal.code(),
        skip.refusal.translation_key(),
    );
    report.push_skipped(detail);
    true
}

struct Record<'a> {
    item: &'a PlannedItem,
    kind: ActionKind,
    size: breathly_platform::ByteSize,
    failure: Option<act::Failure>,
}

fn push(report: &mut CleanupReport, record: &Record<'_>) {
    match record.kind {
        ActionKind::Deleted { to_trash } => report.push_deleted(super::report::Freed {
            id: record.item.id().clone(),
            size: record.size,
            to_trash,
        }),
        ActionKind::Skipped => report.push_skipped(failure_from(record.item, record.failure)),
        ActionKind::Error => report.push_error(failure_from(record.item, record.failure)),
    }
}

fn failure_from(item: &PlannedItem, failure_detail: Option<act::Failure>) -> ItemFailure {
    let detail = failure_detail.unwrap_or(act::Failure {
        code: "clean.removal_failed",
        translation_key: "errors.clean.removal_failed",
    });
    failure(item, detail.code, detail.translation_key)
}

fn failure(item: &PlannedItem, code: &'static str, translation_key: &'static str) -> ItemFailure {
    ItemFailure {
        id: item.id().clone(),
        code,
        translation_key,
    }
}

fn candidate(item: &PlannedItem) -> Candidate<'_> {
    Candidate {
        rule: item.rule(),
        roots: item.roots(),
        path: item.path(),
    }
}

fn intended(risk: RiskLevel) -> Destination {
    match risk {
        RiskLevel::Safe => Destination::Permanent,
        RiskLevel::Review | RiskLevel::Personal => Destination::Trash,
    }
}

fn final_status(report: &CleanupReport) -> CleanupStatus {
    if report.errors.is_empty() && report.skipped.is_empty() {
        CleanupStatus::Completed
    } else {
        CleanupStatus::CompletedWithErrors
    }
}
