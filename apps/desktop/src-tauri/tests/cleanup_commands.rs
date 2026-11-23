//! `start_cleanup` et `stop_cleanup` sans la fenêtre : dossier temporaire, faux de la corbeille.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::SystemTime;

use breathly_desktop_lib::{
    CleanupPorts, CleanupService, DesktopError, ProgressEvent, SummaryStatus,
};
use breathly_disk::{CleanError, SelectedItem};
use breathly_platform::fake::{FakeOpenFiles, FixedClock, FixedStandardPaths, MemoryTrash};
use breathly_platform::{
    ByteSize, Clock, OpenFileInspector, PlatformError, StandardPaths, TrashItem, TrashItemId,
    TrashProvider,
};
use breathly_rules::{Rule, parse_cleanup_rules};

const NOW: SystemTime = SystemTime::UNIX_EPOCH;
const CONTENT: &[u8] = b"contenu";

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

struct World {
    _sandbox: tempfile::TempDir,
    service: CleanupService,
    clock: Arc<FixedClock>,
    gate: Arc<Gate>,
    home: PathBuf,
}

impl World {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sandbox = tempfile::tempdir()?;
        let home = sandbox.path().join("home");
        fs::create_dir_all(home.join(".cache/demo"))?;
        let clock = Arc::new(FixedClock::new(NOW));
        let gate = Arc::new(Gate::new());
        let trash = Arc::new(GatedTrash {
            inner: MemoryTrash::new(),
            gate: Arc::clone(&gate),
            seen: AtomicUsize::new(0),
        });
        let paths: Arc<dyn StandardPaths> = Arc::new(FixedStandardPaths::under(sandbox.path()));
        let open_files: Arc<dyn OpenFileInspector> = Arc::new(FakeOpenFiles::new());
        let service = CleanupService::new(CleanupPorts {
            paths,
            open_files,
            trash,
            clock: Arc::clone(&clock) as Arc<dyn Clock>,
            journal_path: sandbox.path().join("history.jsonl"),
            quarantine_dir: sandbox.path().join("quarantine"),
        });
        Ok(Self {
            _sandbox: sandbox,
            service,
            clock,
            gate,
            home,
        })
    }

    fn file(&self, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = self.home.join(".cache/demo").join(name);
        fs::write(&path, CONTENT)?;
        Ok(path)
    }

    fn item(&self, risk: &str, path: &Path) -> Result<SelectedItem, Box<dyn std::error::Error>> {
        Ok(SelectedItem {
            rule: rule(risk)?,
            roots: vec![self.home.join(".cache/demo")],
            path: path.to_path_buf(),
        })
    }
}

fn rule(risk: &str) -> Result<Rule, Box<dyn std::error::Error>> {
    let source = format!(
        r#"[[rule]]
id = "demo-cache"
name.fr = "Cache de démonstration"
name.en = "Demo cache"
explain.fr = "Fichiers recréés automatiquement."
explain.en = "Files that are re-created automatically."
risk = "{risk}"
os = ["linux", "windows"]
kind = "folder"
paths.linux = ["~/.cache/demo"]
paths.windows = ["%LOCALAPPDATA%/demo"]
"#
    );
    Ok(parse_cleanup_rules(&source)?.remove(0))
}

struct Gate {
    entered: Mutex<bool>,
    release: Mutex<bool>,
    wake: Condvar,
}

impl Gate {
    fn new() -> Self {
        Self {
            entered: Mutex::new(false),
            release: Mutex::new(false),
            wake: Condvar::new(),
        }
    }

    fn wait_entered(&self) {
        let mut entered = self.entered.lock().unwrap_or_else(PoisonError::into_inner);
        while !*entered {
            entered = self
                .wake
                .wait(entered)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    fn release(&self) {
        let mut release = self.release.lock().unwrap_or_else(PoisonError::into_inner);
        *release = true;
        self.wake.notify_all();
    }
}

struct GatedTrash {
    inner: MemoryTrash,
    gate: Arc<Gate>,
    seen: AtomicUsize,
}

impl TrashProvider for GatedTrash {
    fn send(&self, path: &Path) -> Result<TrashItemId, PlatformError> {
        self.hold_first();
        remember(&self.inner, path)?;
        let id = self.inner.send(path)?;
        fs::remove_file(path).map_err(|_| PlatformError::TrashUnavailable)?;
        Ok(id)
    }

    fn list(&self) -> Result<Vec<TrashItem>, PlatformError> {
        self.inner.list()
    }

    fn restore(&self, items: &[TrashItemId]) -> Result<(), PlatformError> {
        self.inner.restore(items)
    }

    fn purge(&self, items: &[TrashItemId]) -> Result<(), PlatformError> {
        self.inner.purge(items)
    }
}

impl GatedTrash {
    fn hold_first(&self) {
        if self.seen.fetch_add(1, Ordering::Relaxed) != 0 {
            return;
        }
        {
            let mut entered = self
                .gate
                .entered
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            *entered = true;
            self.gate.wake.notify_all();
        }
        let mut release = self
            .gate
            .release
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while !*release {
            release = self
                .gate
                .wake
                .wait(release)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

fn remember(trash: &MemoryTrash, path: &Path) -> Result<(), PlatformError> {
    let meta = fs::symlink_metadata(path).map_err(|_| PlatformError::TrashSourceMissing)?;
    trash.add_file(path, ByteSize::from_bytes(meta.len()));
    Ok(())
}
