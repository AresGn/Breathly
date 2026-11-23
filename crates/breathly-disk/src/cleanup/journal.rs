//! Historique minimal avant chaque action (SEC4, ADR 0003). La consultation et l'annulation sont le ticket #90.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use breathly_platform::{ByteSize, limit_to_owner};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::error::CleanError;

const JOURNAL_VERSION: u32 = 1;
const MAX_JOURNAL_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryId(u64);

impl EntryId {
    pub(super) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub enum Destination {
    Trash,
    Permanent,
}

#[derive(Clone, Copy)]
pub enum JournalOutcome {
    Ok,
    Skipped,
    Error,
}

pub struct Pending<'a> {
    pub at: SystemTime,
    pub rule_id: &'a str,
    pub path: &'a Path,
    pub size: ByteSize,
    pub destination: Destination,
}

pub struct Closed {
    pub at: SystemTime,
    pub outcome: JournalOutcome,
    pub trash_ref: Option<String>,
}

pub trait CleanupJournal {
    /// Écrit l'entrée `pending` et la synchronise avant toute suppression.
    ///
    /// # Errors
    /// `HistoryUnwritable` : l'appelant ne doit rien supprimer.
    fn record_pending(&mut self, pending: Pending<'_>) -> Result<EntryId, CleanError>;

    /// Inscrit le résultat d'une entrée déjà `pending`.
    ///
    /// # Errors
    /// `HistoryUnwritable`.
    fn record_outcome(&mut self, id: EntryId, closed: Closed) -> Result<(), CleanError>;

    /// Inscrit un élément ignoré ou en erreur, sans suppression.
    ///
    /// # Errors
    /// `HistoryUnwritable` : le nettoyage s'arrête avant l'élément suivant.
    fn record_note(
        &mut self,
        pending: Pending<'_>,
        outcome: JournalOutcome,
    ) -> Result<(), CleanError>;
}

/// Fichier JSON Lines de l'ADR 0003, borné, réservé au propriétaire.
pub struct JsonlJournal {
    file: File,
    next_id: u64,
}

impl JsonlJournal {
    /// Ouvre ou crée le fichier. Une ligne illisible ou un fichier trop gros est un refus (SEC5).
    ///
    /// # Errors
    /// `HistoryUnwritable`.
    pub fn open(path: &Path) -> Result<Self, CleanError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| CleanError::HistoryUnwritable)?;
            limit_to_owner(parent).map_err(|_| CleanError::HistoryUnwritable)?;
        }
        let mut file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| CleanError::HistoryUnwritable)?;
        limit_to_owner(path).map_err(|_| CleanError::HistoryUnwritable)?;
        let next_id = next_identifier(&mut file)?;
        Ok(Self { file, next_id })
    }

    fn allocate(&mut self) -> Result<u64, CleanError> {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(CleanError::HistoryUnwritable)?;
        Ok(id)
    }

    fn write_line(&mut self, value: &serde_json::Value) -> Result<(), CleanError> {
        self.file
            .lock()
            .map_err(|_| CleanError::HistoryUnwritable)?;
        let wrote = writeln!(self.file, "{value}").and_then(|()| self.file.sync_all());
        let unlocked = self.file.unlock();
        wrote
            .and(unlocked)
            .map_err(|_| CleanError::HistoryUnwritable)
    }
}

impl CleanupJournal for JsonlJournal {
    fn record_pending(&mut self, pending: Pending<'_>) -> Result<EntryId, CleanError> {
        let id = self.allocate()?;
        self.write_line(&entry_line(id, &pending, "pending")?)?;
        Ok(EntryId(id))
    }

    fn record_outcome(&mut self, id: EntryId, closed: Closed) -> Result<(), CleanError> {
        self.write_line(&outcome_line(id.0, &closed)?)
    }

    fn record_note(
        &mut self,
        pending: Pending<'_>,
        outcome: JournalOutcome,
    ) -> Result<(), CleanError> {
        let id = self.allocate()?;
        let name = outcome_name(outcome);
        self.write_line(&entry_line(id, &pending, name)?)
    }
}

fn next_identifier(file: &mut File) -> Result<u64, CleanError> {
    let length = file
        .metadata()
        .map_err(|_| CleanError::HistoryUnwritable)?
        .len();
    if length > MAX_JOURNAL_BYTES {
        return Err(CleanError::HistoryUnwritable);
    }
    let mut text = String::new();
    file.seek(SeekFrom::Start(0))
        .map_err(|_| CleanError::HistoryUnwritable)?;
    file.read_to_string(&mut text)
        .map_err(|_| CleanError::HistoryUnwritable)?;
    max_id(&text)?
        .checked_add(1)
        .ok_or(CleanError::HistoryUnwritable)
}

fn max_id(text: &str) -> Result<u64, CleanError> {
    let mut max = 0_u64;
    for raw in text.lines().filter(|line| !line.is_empty()) {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|_| CleanError::HistoryUnwritable)?;
        let id = value
            .get("id")
            .and_then(serde_json::Value::as_u64)
            .ok_or(CleanError::HistoryUnwritable)?;
        max = max.max(id);
    }
    Ok(max)
}

fn entry_line(
    id: u64,
    pending: &Pending<'_>,
    outcome: &str,
) -> Result<serde_json::Value, CleanError> {
    Ok(serde_json::json!({
        "v": JOURNAL_VERSION,
        "id": id,
        "ts": stamp(pending.at)?,
        "action": "clean",
        "rule_id": pending.rule_id,
        "path": pending.path.to_string_lossy(),
        "size_bytes": pending.size.bytes(),
        "destination": destination_name(pending.destination),
        "trash_ref": Option::<&str>::None,
        "outcome": outcome,
    }))
}

fn outcome_line(id: u64, closed: &Closed) -> Result<serde_json::Value, CleanError> {
    Ok(serde_json::json!({
        "v": JOURNAL_VERSION,
        "id": id,
        "ts": stamp(closed.at)?,
        "action": "clean",
        "outcome": outcome_name(closed.outcome),
        "trash_ref": closed.trash_ref,
    }))
}

fn destination_name(destination: Destination) -> &'static str {
    match destination {
        Destination::Trash => "trash",
        Destination::Permanent => "permanent",
    }
}

fn outcome_name(outcome: JournalOutcome) -> &'static str {
    match outcome {
        JournalOutcome::Ok => "ok",
        JournalOutcome::Skipped => "skipped",
        JournalOutcome::Error => "error",
    }
}

fn stamp(at: SystemTime) -> Result<String, CleanError> {
    let since = at
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CleanError::HistoryUnwritable)?;
    let seconds = i64::try_from(since.as_secs()).map_err(|_| CleanError::HistoryUnwritable)?;
    let date = OffsetDateTime::from_unix_timestamp(seconds)
        .map_err(|_| CleanError::HistoryUnwritable)?
        + time::Duration::nanoseconds(i64::from(since.subsec_nanos()));
    date.format(&Rfc3339)
        .map_err(|_| CleanError::HistoryUnwritable)
}
