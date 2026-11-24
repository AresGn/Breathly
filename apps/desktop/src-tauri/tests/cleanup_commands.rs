//! `start_cleanup` et `stop_cleanup` sans la fenêtre : dossier temporaire, faux de la corbeille.

#[path = "cleanup_commands/world.rs"]
mod world;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use breathly_desktop_lib::{DesktopError, ProgressEvent, SummaryStatus};
use breathly_disk::CleanError;
use world::{CONTENT, PanickingTrash, World};

#[test]
fn an_invalid_or_expired_token_changes_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let file = world.file("old.bin")?;
    let error = world
        .service
        .start("nope", Box::new(|_| {}))
        .expect_err("jeton refusé");
    assert_eq!(error, DesktopError::Clean(CleanError::TokenInvalid));
    let confirmation = world
        .service
        .issue_confirmation(vec![world.item("safe", &file)?])?;
    world.clock.advance(breathly_disk::CONFIRMATION_TTL);
    let expired = world
        .service
        .start(&confirmation, Box::new(|_| {}))
        .expect_err("expiré");
    assert_eq!(expired, DesktopError::Clean(CleanError::TokenExpired));
    assert_eq!(fs::read(file)?, CONTENT);
    Ok(())
}

#[test]
fn progress_is_per_item_and_stop_reports_both_numbers() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let first = world.file("a.bin")?;
    let second = world.file("b.bin")?;
    let confirmation = world.service.issue_confirmation(vec![
        world.item("review", &first)?,
        world.item("review", &second)?,
    ])?;
    let (events, sink) = collect();
    let id = world.service.start(&confirmation, sink).expect("départ");
    world.gate.wait_entered();
    world.service.request_stop(&id).expect("arrêt");
    world.gate.release();
    let summary = world.service.wait_report(&id).expect("résumé");
    assert_eq!(summary.status, SummaryStatus::Stopped);
    assert_eq!(summary.done, 1);
    assert_eq!(summary.not_done, 1);
    assert_eq!(summary.trashed_bytes, 7);
    assert_eq!(summary.freed_now_bytes, 0);
    assert!(!first.exists());
    assert!(second.exists());
    let seen = events.lock().unwrap_or_else(PoisonError::into_inner);
    assert!(
        seen.iter()
            .any(|event| matches!(event, ProgressEvent::Step { done: 1, .. }))
    );
    assert!(
        seen.iter()
            .any(|event| matches!(event, ProgressEvent::Cancelled))
    );
    Ok(())
}

#[test]
fn a_second_start_is_refused_and_keeps_its_token() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let first = world.file("a.bin")?;
    let second = world.file("b.bin")?;
    let running = world
        .service
        .issue_confirmation(vec![world.item("review", &first)?])?;
    let held = world
        .service
        .issue_confirmation(vec![world.item("safe", &second)?])?;
    let id = world
        .service
        .start(&running, Box::new(|_| {}))
        .expect("départ");
    world.gate.wait_entered();
    let refused = world
        .service
        .start(&held, Box::new(|_| {}))
        .expect_err("occupé");
    assert_eq!(refused, DesktopError::AlreadyRunning);
    world.service.request_stop(&id).expect("arrêt");
    world.gate.release();
    world.service.wait_report(&id).expect("fin");
    let again = world
        .service
        .start(&held, Box::new(|_| {}))
        .expect("le jeton tient");
    world.service.wait_report(&again).expect("second");
    assert!(!second.exists());
    Ok(())
}

#[test]
fn the_capability_allows_exactly_the_declared_commands() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = fs::read_to_string(root.join("src/lib.rs"))?;
    let capability = fs::read_to_string(root.join("capabilities/main.json"))?;
    let files = fs::read_dir(root.join("capabilities"))?.count();
    assert_eq!(files, 1);
    for name in breathly_desktop_lib::COMMANDS {
        assert!(lib.contains(name), "{name}");
        let permission = format!("allow-{}", name.replace('_', "-"));
        assert!(capability.contains(&permission), "{permission}");
    }
    Ok(())
}

fn collect() -> (
    Arc<Mutex<Vec<ProgressEvent>>>,
    breathly_desktop_lib::ProgressSink,
) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let shared = Arc::clone(&events);
    let sink = Box::new(move |event| {
        shared
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(event);
    });
    (events, sink)
}

#[test]
fn a_used_token_is_refused_and_changes_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let file = world.file("old.bin")?;
    let confirmation = world
        .service
        .issue_confirmation(vec![world.item("safe", &file)?])?;
    let id = world
        .service
        .start(&confirmation, Box::new(|_| {}))
        .expect("départ");
    world.service.wait_report(&id).expect("fin");
    let again = world
        .service
        .start(&confirmation, Box::new(|_| {}))
        .expect_err("déjà utilisé");
    assert_eq!(again, DesktopError::Clean(CleanError::TokenUsed));
    Ok(())
}

#[test]
fn the_summary_lists_deleted_skipped_and_failed_items() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::new()?;
    let cache = world.file("old.bin")?;
    let outside = world.home.join("Documents/keep.txt");
    fs::create_dir_all(outside.parent().expect("dossier"))?;
    fs::write(&outside, CONTENT)?;
    let confirmation = world.service.issue_confirmation(vec![
        world.item("safe", &cache)?,
        world.item("review", &outside)?,
    ])?;
    let (events, sink) = collect();
    let id = world.service.start(&confirmation, sink).expect("départ");
    let summary = world.service.wait_report(&id).expect("fin");
    assert_eq!(summary.status, SummaryStatus::CompletedWithErrors);
    assert_eq!(summary.deleted.len(), 1);
    assert_eq!(summary.skipped.len(), 1);
    assert!(summary.errors.is_empty());
    assert_eq!(summary.freed_now_bytes, 7);
    assert!(outside.exists());
    let seen = events.lock().unwrap_or_else(PoisonError::into_inner);
    assert_eq!(seen.first(), Some(&ProgressEvent::Started));
    assert_eq!(seen.last(), Some(&ProgressEvent::Finished));
    Ok(())
}

#[test]
fn a_panic_keeps_what_was_already_done() -> Result<(), Box<dyn std::error::Error>> {
    let world = World::with_trash(|_| Arc::new(PanickingTrash))?;
    let cache = world.file("old.bin")?;
    let review = world.file("b.bin")?;
    let confirmation = world.service.issue_confirmation(vec![
        world.item("safe", &cache)?,
        world.item("review", &review)?,
    ])?;
    let (events, sink) = collect();
    let id = world.service.start(&confirmation, sink).expect("départ");
    let summary = world.service.wait_report(&id).expect("fin");
    assert_eq!(summary.status, SummaryStatus::Panicked);
    assert_eq!(summary.done, 1);
    assert_eq!(summary.not_done, 1);
    assert_eq!(summary.freed_now_bytes, 7);
    assert_eq!(summary.deleted.len(), 1);
    assert!(!cache.exists());
    let seen = events.lock().unwrap_or_else(PoisonError::into_inner);
    assert!(matches!(seen.last(), Some(ProgressEvent::Failed { .. })));
    Ok(())
}
