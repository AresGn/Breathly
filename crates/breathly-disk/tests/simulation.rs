//! TS-04 et US-032 : la simulation ne modifie rien, ses chiffres sont ceux du nettoyage, son jeton est à usage
//! unique, expirant et lié à sa sélection. Dossier temporaire et faux de `breathly-platform` seulement.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use breathly_disk::{
    CONFIRMATION_TTL, CleanError, ConfirmationStore, Destination, Foreseen, Guardrails,
    SelectedItem, Simulation, SimulationRequest, simulate,
};
use breathly_platform::fake::FixedClock;
use breathly_platform::{ByteSize, Clock};

mod common;
use common::{Fixture, TestResult, rule};

const NOW: SystemTime = SystemTime::UNIX_EPOCH;

fn selected(
    fixture: &Fixture,
    risk: &str,
    path: &Path,
) -> Result<SelectedItem, Box<dyn std::error::Error>> {
    Ok(SelectedItem {
        rule: rule(risk)?,
        roots: vec![fixture.home().join(".cache/demo")],
        path: path.to_path_buf(),
    })
}

fn run(
    fixture: &Fixture,
    store: &ConfirmationStore,
    items: Vec<SelectedItem>,
) -> Result<Simulation, CleanError> {
    let guardrails = Guardrails::new(&fixture.paths, &fixture.open_files);
    simulate(SimulationRequest {
        items,
        guardrails: &guardrails,
        store,
        now: NOW,
    })
}

fn tree(root: &Path) -> Result<BTreeSet<(PathBuf, u64)>, Box<dyn std::error::Error>> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            seen.insert((entry.path(), meta.len()));
            if meta.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    Ok(seen)
}

#[test]
fn simulating_changes_nothing_on_disk() -> TestResult {
    let fixture = Fixture::new()?;
    let cache = fixture.file(".cache/demo/a.bin")?;
    let review = fixture.file(".cache/demo/b.bin")?;
    let before = tree(fixture.sandbox.path())?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    run(
        &fixture,
        &store,
        vec![
            selected(&fixture, "safe", &cache)?,
            selected(&fixture, "review", &review)?,
        ],
    )?;
    assert_eq!(tree(fixture.sandbox.path())?, before);
    assert!(!fixture.sandbox.path().join("history.jsonl").exists());
    Ok(())
}

#[test]
fn the_report_gives_both_numbers_and_a_reason_for_each_skip() -> TestResult {
    let fixture = Fixture::new()?;
    let cache = fixture.file(".cache/demo/a.bin")?;
    let review = fixture.file(".cache/demo/b.bin")?;
    let outside = fixture.file("Documents/keep.txt")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let simulation = run(
        &fixture,
        &store,
        vec![
            selected(&fixture, "safe", &cache)?,
            selected(&fixture, "review", &review)?,
            selected(&fixture, "review", &outside)?,
            selected(&fixture, "personal", &review)?,
        ],
    )?;
    let report = &simulation.report;
    assert_eq!(report.freed_now, ByteSize::from_bytes(7));
    assert_eq!(report.trashed, ByteSize::from_bytes(7));
    let found: Vec<Foreseen> = report.items.iter().map(|item| item.foreseen).collect();
    assert_acts(&found);
    assert_skips(&found);
    Ok(())
}

fn assert_skips(found: &[Foreseen]) {
    assert!(matches!(found[2], Foreseen::Skip(reason) if reason.code == "guard.outside_rule"));
    assert!(matches!(found[3], Foreseen::Skip(reason) if reason.code == "guard.not_selectable"));
}

fn assert_acts(found: &[Foreseen]) {
    assert_eq!(
        found[0],
        Foreseen::Act {
            destination: Destination::Permanent,
            size: ByteSize::from_bytes(7)
        }
    );
    assert_eq!(
        found[1],
        Foreseen::Act {
            destination: Destination::Trash,
            size: ByteSize::from_bytes(7)
        }
    );
}

#[test]
fn a_folder_is_measured_by_the_sum_of_its_files() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.file(".cache/demo/dir/one.bin")?;
    fixture.file(".cache/demo/dir/two.bin")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let dir = fixture.home().join(".cache/demo/dir");
    let simulation = run(&fixture, &store, vec![selected(&fixture, "safe", &dir)?])?;
    assert_eq!(simulation.report.freed_now, ByteSize::from_bytes(14));
    Ok(())
}

/// US-032.2 (« identique au nettoyage ») n'est pas couvert ici : il attend l'exécuteur (#80), qui lira la même
/// règle (`foresee`). Ce test garantit seulement qu'une simulation est reproductible.
#[test]
fn a_simulation_is_repeatable_on_the_same_disk_state() -> TestResult {
    let fixture = Fixture::new()?;
    let cache = fixture.file(".cache/demo/a.bin")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let first = run(&fixture, &store, vec![selected(&fixture, "safe", &cache)?])?;
    let second = run(&fixture, &store, vec![selected(&fixture, "safe", &cache)?])?;
    let foreseen = |simulation: &Simulation| {
        simulation
            .report
            .items
            .iter()
            .map(|item| item.foreseen)
            .collect::<Vec<_>>()
    };
    assert_eq!(foreseen(&first), foreseen(&second));
    assert_eq!(first.report.freed_now, second.report.freed_now);
    Ok(())
}

#[test]
fn an_empty_selection_is_refused_and_issues_no_token() -> TestResult {
    let fixture = Fixture::new()?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let error = run(&fixture, &store, Vec::new()).expect_err("vide");
    assert_eq!(error, CleanError::EmptySelection);
    Ok(())
}

#[test]
fn a_token_works_once_and_only_for_its_own_selection() -> TestResult {
    let fixture = Fixture::new()?;
    let first_file = fixture.file(".cache/demo/a.bin")?;
    let second_file = fixture.file(".cache/demo/b.bin")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let first = run(
        &fixture,
        &store,
        vec![selected(&fixture, "safe", &first_file)?],
    )?;
    let second = run(
        &fixture,
        &store,
        vec![selected(&fixture, "safe", &second_file)?],
    )?;
    let plan = store.consume(&first.confirmation, NOW)?;
    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].id(), &first.report.items[0].id);
    assert_ne!(plan[0].id(), &second.report.items[0].id);
    assert_eq!(plan[0].path(), first_file);
    assert_eq!(
        store.consume(&first.confirmation, NOW).expect_err("rejoué"),
        CleanError::TokenUsed
    );
    assert_eq!(store.consume(&second.confirmation, NOW)?.len(), 1);
    Ok(())
}

#[test]
fn an_expired_token_is_refused_by_the_clock() -> TestResult {
    let fixture = Fixture::new()?;
    let file = fixture.file(".cache/demo/a.bin")?;
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    let simulation = run(&fixture, &store, vec![selected(&fixture, "safe", &file)?])?;
    let clock = FixedClock::new(NOW);
    clock.advance(CONFIRMATION_TTL);
    let error = store
        .consume(&simulation.confirmation, clock.now())
        .expect_err("expiré");
    assert_eq!(error, CleanError::TokenExpired);
    assert!(file.exists());
    Ok(())
}

#[test]
fn a_forged_or_malformed_token_is_invalid() {
    let store = ConfirmationStore::new(CONFIRMATION_TTL);
    assert_eq!(
        breathly_disk::ConfirmationToken::parse("../../etc").expect_err("forme"),
        CleanError::TokenInvalid
    );
    let forged = breathly_disk::ConfirmationToken::parse(&"a".repeat(32)).expect("forme valide");
    assert_eq!(
        store.consume(&forged, NOW).expect_err("inconnu"),
        CleanError::TokenInvalid
    );
}
