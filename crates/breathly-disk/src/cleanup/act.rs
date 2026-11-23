//! Une action sur un élément déjà revérifié : corbeille ou suppression définitive (ADR 0005, ADR 0018).

use std::ffi::OsString;
use std::path::Path;
use std::time::SystemTime;

use breathly_platform::{
    ByteSize, CanonicalPath, PathAnchor, PlatformError, RemovalError, TrashProvider,
};
use breathly_rules::RiskLevel;

use super::journal::{CleanupJournal, Closed, Destination, EntryId, JournalOutcome, Pending};
use crate::{Refusal, VerifiedItem};

pub(super) struct Action {
    pub kind: ActionKind,
    pub size: ByteSize,
    pub failure: Option<Failure>,
}

#[derive(Clone, Copy)]
pub(super) enum ActionKind {
    Deleted { to_trash: bool },
    Skipped,
    Error,
}

#[derive(Clone, Copy)]
pub(super) struct Failure {
    pub code: &'static str,
    pub translation_key: &'static str,
}

pub(super) struct ActionIo<'a> {
    pub trash: &'a dyn TrashProvider,
    pub journal: &'a mut dyn CleanupJournal,
    pub quarantine: &'a Path,
    pub now: SystemTime,
}

pub(super) enum Perform {
    Continue(Action),
    Halt(Option<Action>),
}

pub(super) fn perform(verified: &VerifiedItem, io: &mut ActionIo<'_>) -> Perform {
    let size = match verified.anchor().byte_size() {
        Ok(size) => size,
        Err(RemovalError::CrossVolume) => {
            return note(
                io,
                &skipped(verified, failure_of(RemovalError::CrossVolume)),
            );
        }
        Err(error) => return note(io, &failed_note(verified, failure_of(error))),
    };
    let Some(destination) = destination_of(verified.risk()) else {
        return note(io, &skipped(verified, not_selectable()));
    };
    let input = PendingInput {
        verified,
        at: io.now,
        size,
        destination,
    };
    let Ok(entry) = io.journal.record_pending(pending(&input)) else {
        return Perform::Halt(None);
    };
    let step = match destination {
        Destination::Permanent => delete(verified).map(|()| None),
        Destination::Trash => send_to_trash(verified, io, entry),
    };
    close(
        io,
        Close {
            entry,
            size,
            destination,
            step,
        },
    )
}

struct Close {
    entry: EntryId,
    size: ByteSize,
    destination: Destination,
    step: Result<Option<String>, StepFailure>,
}

fn close(io: &mut ActionIo<'_>, close: Close) -> Perform {
    let (action, closed) = match close.step {
        Ok(trash_ref) => (
            deleted(close.size, close.destination),
            ok_closed(io.now, trash_ref),
        ),
        Err(StepFailure::Skip(failure)) => {
            (failed(ActionKind::Skipped, failure), skip_closed(io.now))
        }
        Err(StepFailure::Fail(failure)) => (failed(ActionKind::Error, failure), err_closed(io.now)),
    };
    if io.journal.record_outcome(close.entry, closed).is_err() {
        Perform::Halt(Some(action))
    } else {
        Perform::Continue(action)
    }
}

fn send_to_trash(
    verified: &VerifiedItem,
    io: &mut ActionIo<'_>,
    entry: EntryId,
) -> Result<Option<String>, StepFailure> {
    let stored = OsString::from(format!("q-{}", entry.raw()));
    if let Err(error) = verified.anchor().relocate(io.quarantine, &stored) {
        return Err(relocate_failure(error));
    }
    match io.trash.send(&io.quarantine.join(&stored)) {
        Ok(id) => Ok(Some(id.as_str().to_owned())),
        Err(error) => Err(trash_failure(
            io,
            &TrashMiss {
                verified,
                stored: &stored,
                error: &error,
            },
        )),
    }
}

struct TrashMiss<'a> {
    verified: &'a VerifiedItem,
    stored: &'a OsString,
    error: &'a PlatformError,
}

fn trash_failure(io: &ActionIo<'_>, miss: &TrashMiss<'_>) -> StepFailure {
    if put_back(io.quarantine, miss.stored, miss.verified.origin()).is_err() {
        return StepFailure::Fail(stuck());
    }
    StepFailure::Fail(Failure {
        code: miss.error.code(),
        translation_key: miss.error.translation_key(),
    })
}

fn put_back(quarantine: &Path, stored: &OsString, origin: &Path) -> Result<(), RemovalError> {
    let parent = origin.parent().ok_or(RemovalError::Failed)?;
    let name = origin.file_name().ok_or(RemovalError::Failed)?;
    let root = CanonicalPath::resolve(quarantine).map_err(|_| RemovalError::Failed)?;
    let target =
        CanonicalPath::resolve(&quarantine.join(stored)).map_err(|_| RemovalError::Failed)?;
    let anchor = PathAnchor::open(&root, &target).map_err(|_| RemovalError::Failed)?;
    anchor.relocate(parent, name)
}

fn delete(verified: &VerifiedItem) -> Result<(), StepFailure> {
    verified
        .anchor()
        .delete_permanently()
        .map_err(|error| StepFailure::Fail(failure_of(error)))
}

struct Note<'a> {
    verified: &'a VerifiedItem,
    failure: Failure,
    outcome: JournalOutcome,
}

fn skipped(verified: &VerifiedItem, failure: Failure) -> Note<'_> {
    Note {
        verified,
        failure,
        outcome: JournalOutcome::Skipped,
    }
}

fn failed_note(verified: &VerifiedItem, failure: Failure) -> Note<'_> {
    Note {
        verified,
        failure,
        outcome: JournalOutcome::Error,
    }
}

fn note(io: &mut ActionIo<'_>, note: &Note<'_>) -> Perform {
    let kind = match note.outcome {
        JournalOutcome::Skipped => ActionKind::Skipped,
        JournalOutcome::Error | JournalOutcome::Ok => ActionKind::Error,
    };
    let action = failed(kind, note.failure);
    let destination = destination_of(note.verified.risk()).unwrap_or(Destination::Trash);
    let input = PendingInput {
        verified: note.verified,
        at: io.now,
        size: ByteSize::from_bytes(0),
        destination,
    };
    let pending = pending(&input);
    if io.journal.record_note(pending, note.outcome).is_err() {
        Perform::Halt(Some(action))
    } else {
        Perform::Continue(action)
    }
}

struct PendingInput<'a> {
    verified: &'a VerifiedItem,
    at: SystemTime,
    size: ByteSize,
    destination: Destination,
}

fn pending<'a>(input: &'a PendingInput<'a>) -> Pending<'a> {
    Pending {
        at: input.at,
        rule_id: input.verified.rule().as_str(),
        path: input.verified.origin(),
        size: input.size,
        destination: input.destination,
    }
}

fn destination_of(risk: RiskLevel) -> Option<Destination> {
    match risk {
        RiskLevel::Safe => Some(Destination::Permanent),
        RiskLevel::Review => Some(Destination::Trash),
        RiskLevel::Personal => None,
    }
}

fn relocate_failure(error: RemovalError) -> StepFailure {
    match error {
        RemovalError::CrossVolume => StepFailure::Skip(failure_of(error)),
        RemovalError::Occupied | RemovalError::Failed => StepFailure::Fail(failure_of(error)),
    }
}

enum StepFailure {
    Skip(Failure),
    Fail(Failure),
}

fn deleted(size: ByteSize, destination: Destination) -> Action {
    Action {
        kind: ActionKind::Deleted {
            to_trash: matches!(destination, Destination::Trash),
        },
        size,
        failure: None,
    }
}

fn failed(kind: ActionKind, failure: Failure) -> Action {
    Action {
        kind,
        size: ByteSize::from_bytes(0),
        failure: Some(failure),
    }
}

fn ok_closed(at: SystemTime, trash_ref: Option<String>) -> Closed {
    Closed {
        at,
        outcome: JournalOutcome::Ok,
        trash_ref,
    }
}

fn skip_closed(at: SystemTime) -> Closed {
    Closed {
        at,
        outcome: JournalOutcome::Skipped,
        trash_ref: None,
    }
}

fn err_closed(at: SystemTime) -> Closed {
    Closed {
        at,
        outcome: JournalOutcome::Error,
        trash_ref: None,
    }
}

fn failure_of(error: RemovalError) -> Failure {
    Failure {
        code: error.code(),
        translation_key: error.translation_key(),
    }
}

fn not_selectable() -> Failure {
    Failure {
        code: Refusal::NotSelectable.code(),
        translation_key: Refusal::NotSelectable.translation_key(),
    }
}

fn stuck() -> Failure {
    Failure {
        code: "clean.quarantine_stuck",
        translation_key: "errors.clean.quarantine_stuck",
    }
}
