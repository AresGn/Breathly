//! Nettoyage dans un dossier temporaire, avec le faux `TrashProvider` (US-030, US-033, US-036, SEC3, SEC4).

#[path = "cleanup/adapters.rs"]
mod adapters;
#[path = "common/mod.rs"]
mod common;
#[path = "cleanup/world.rs"]
mod world;
use world::{Attempt, BrokenJournal, Heard, Noted, WatchingJournal, World, plan};

use std::fs;
use std::time::SystemTime;

use adapters::{StopAfterFirst, Unavailable};
use breathly_disk::{
    CONFIRMATION_TTL, CleanError, CleanupReport, CleanupStatus, ConfirmationStore,
    ConfirmationToken, JsonlJournal, ProgressMessage, StopFlag,
};
use breathly_platform::fake::FixedClock;
use breathly_platform::{Clock, TrashProvider};

const NOW: SystemTime = SystemTime::UNIX_EPOCH;
const CONTENT: &[u8] = b"contenu";

#[test]
fn an_invalid_expired_or_replayed_token_changes_nothing() -> Result<(), Box<dyn std::error::Error>>
{
    let world = World::new()?;
    let file = world.cache_file("old.bin")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let unknown = ConfirmationToken::parse("0123456789abcdef0123456789abcdef")?;
    assert_eq!(
        store.consume(&unknown, NOW).unwrap_err(),
        CleanError::TokenInvalid
    );
    assert_eq!(
        ConfirmationToken::parse("ab").unwrap_err(),
        CleanError::TokenInvalid
    );
    let confirmation = store.issue(vec![world.item("safe", &file)?], NOW)?;
    let clock = FixedClock::new(NOW);
    clock.advance(CONFIRMATION_TTL);
    assert_eq!(
        store.consume(&confirmation, clock.now()).unwrap_err(),
        CleanError::TokenExpired
    );
    let again = store.issue(vec![world.item("safe", &file)?], NOW)?;
    store.consume(&again, NOW)?;
    assert_eq!(
        store.consume(&again, NOW).unwrap_err(),
        CleanError::TokenUsed
    );
    assert_eq!(format!("{again:?}"), "ConfirmationToken(..)");
    assert_eq!(fs::read(file)?, CONTENT);
    Ok(())
}

#[test]
fn a_token_only_cleans_its_own_plan() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let kept = world.cache_file("kept.bin")?;
    let gone = world.cache_file("gone.bin")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let _other = store.issue(vec![world.item("safe", &kept)?], NOW)?;
    let confirmation = store.issue(vec![world.item("safe", &gone)?], NOW)?;
    let plan = store.consume(&confirmation, NOW)?;
    assert!(!format!("{plan:?}").contains("gone.bin"));
    let bridge = world.bridge();
    let report = world.clean(&Attempt {
        plan: &plan,
        stop: &StopFlag::new(),
        trash: &bridge,
    })?;
    assert_eq!(report.status, CleanupStatus::Completed);
    assert_eq!(report.freed_now.bytes(), 7);
    assert_eq!(report.trashed.bytes(), 0);
    assert_eq!(report.deleted.len(), 1);
    assert!(!gone.exists());
    assert_eq!(fs::read(kept)?, CONTENT);
    Ok(())
}

#[test]
fn a_review_item_goes_to_the_trash_and_a_refusal_is_skipped()
-> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let review = world.cache_file("notes.bin")?;
    let personal = world.cache_file("photo.bin")?;
    let plan = plan(vec![
        world.item("review", &review)?,
        world.item("personal", &personal)?,
    ])?;
    let bridge = world.bridge();
    let mut steps = Vec::new();
    let mut hear = |step: ProgressMessage| steps.push(step.done);
    let report = world.clean_progress(&mut Heard {
        plan: &plan,
        trash: &bridge,
        progress: &mut hear,
    })?;
    assert_review(&report, &steps);
    assert_eq!(world.trash.list()?.len(), 1);
    assert!(!review.exists());
    assert!(personal.exists());
    Ok(())
}

fn assert_review(report: &CleanupReport, steps: &[u32]) {
    assert_eq!(report.status, CleanupStatus::CompletedWithErrors);
    assert_eq!(report.trashed.bytes(), 7);
    assert_eq!(report.freed_now.bytes(), 0);
    assert_eq!(report.deleted.len(), 1);
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].code, "guard.not_selectable");
    assert_eq!(steps, [1, 2]);
}

#[test]
fn an_error_on_one_item_does_not_stop_the_next() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let first = world.cache_file("a.bin")?;
    let second = world.cache_file("b.bin")?;
    let plan = plan(vec![
        world.item("review", &first)?,
        world.item("safe", &second)?,
    ])?;
    let report = world.clean(&Attempt {
        plan: &plan,
        stop: &StopFlag::new(),
        trash: &Unavailable,
    })?;
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.deleted.len(), 1);
    assert_eq!(report.freed_now.bytes(), 7);
    assert_eq!(fs::read(&first)?, CONTENT);
    assert!(!second.exists());
    Ok(())
}

#[test]
fn stop_finishes_the_current_item_and_counts_the_rest() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let first = world.cache_file("a.bin")?;
    let second = world.cache_file("b.bin")?;
    let plan = plan(vec![
        world.item("review", &first)?,
        world.item("review", &second)?,
    ])?;
    let stop = StopFlag::new();
    let stopping = StopAfterFirst::new(world.bridge(), &stop);
    let report = world.clean(&Attempt {
        plan: &plan,
        stop: &stop,
        trash: &stopping,
    })?;
    assert_eq!(report.status, CleanupStatus::Stopped);
    assert_eq!(report.done, 1);
    assert_eq!(report.not_done, 1);
    assert!(!first.exists());
    assert!(second.exists());
    Ok(())
}

#[test]
fn an_unwritable_history_deletes_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let file = world.cache_file("old.bin")?;
    let plan = plan(vec![world.item("safe", &file)?])?;
    let mut journal = BrokenJournal;
    let bridge = world.bridge();
    let report = world.clean_journal(&mut Noted {
        plan: &plan,
        trash: &bridge,
        journal: &mut journal,
    })?;
    assert_eq!(report.status, CleanupStatus::Aborted);
    assert_eq!(report.done, 0);
    assert_eq!(fs::read(file)?, CONTENT);
    Ok(())
}

#[test]
fn the_pending_line_is_written_before_the_file_disappears() -> Result<(), Box<dyn std::error::Error>>
{
    let world = World::new()?;
    let file = world.cache_file("old.bin")?;
    let plan = plan(vec![world.item("safe", &file)?])?;
    let path = world.sandbox().join("history.jsonl");
    let mut journal = WatchingJournal {
        inner: JsonlJournal::open(&path)?,
        saw_file: false,
    };
    let bridge = world.bridge();
    world.clean_journal(&mut Noted {
        plan: &plan,
        trash: &bridge,
        journal: &mut journal,
    })?;
    assert!(journal.saw_file);
    assert!(!file.exists());
    let text = fs::read_to_string(path)?;
    let pending = text.lines().next().ok_or("historique vide")?;
    assert!(pending.contains("\"outcome\":\"pending\""));
    assert!(text.contains("\"outcome\":\"ok\""));
    Ok(())
}
