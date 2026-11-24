//! Monde de test des commandes de nettoyage : dossier temporaire, fausse horloge, corbeille factice.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::SystemTime;

use breathly_desktop_lib::{CleanupPorts, CleanupService};
use breathly_disk::SelectedItem;
use breathly_platform::fake::{FakeOpenFiles, FixedClock, FixedStandardPaths, MemoryTrash};
use breathly_platform::{
    ByteSize, Clock, OpenFileInspector, PlatformError, StandardPaths, TrashItem, TrashItemId,
    TrashProvider,
};
use breathly_rules::{Rule, parse_cleanup_rules};

const NOW: SystemTime = SystemTime::UNIX_EPOCH;
pub const CONTENT: &[u8] = b"contenu";

pub struct World {
    _sandbox: tempfile::TempDir,
    pub service: CleanupService,
    pub clock: Arc<FixedClock>,
    pub gate: Arc<Gate>,
    pub home: PathBuf,
}

impl World {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_trash(|gate| {
            Arc::new(GatedTrash {
                inner: MemoryTrash::new(),
                gate,
                seen: AtomicUsize::new(0),
            })
        })
    }

    pub fn with_trash(
        make: impl FnOnce(Arc<Gate>) -> Arc<dyn TrashProvider>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let sandbox = tempfile::tempdir()?;
        let home = sandbox.path().join("home");
        fs::create_dir_all(home.join(".cache/demo"))?;
        let clock = Arc::new(FixedClock::new(NOW));
        let gate = Arc::new(Gate::new());
        let trash = make(Arc::clone(&gate));
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

    pub fn file(&self, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = self.home.join(".cache/demo").join(name);
        fs::write(&path, CONTENT)?;
        Ok(path)
    }

    pub fn item(
        &self,
        risk: &str,
        path: &Path,
    ) -> Result<SelectedItem, Box<dyn std::error::Error>> {
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

pub struct Gate {
    entered: Mutex<bool>,
    release: Mutex<bool>,
    wake: Condvar,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            entered: Mutex::new(false),
            release: Mutex::new(false),
            wake: Condvar::new(),
        }
    }

    pub fn wait_entered(&self) {
        let mut entered = self.entered.lock().unwrap_or_else(PoisonError::into_inner);
        while !*entered {
            entered = self
                .wake
                .wait(entered)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    pub fn release(&self) {
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

/// Une corbeille qui échoue par une panique : le nettoyage doit quand même rendre un résumé vrai.
pub struct PanickingTrash;

impl TrashProvider for PanickingTrash {
    fn send(&self, _path: &Path) -> Result<TrashItemId, PlatformError> {
        panic!("corbeille cassée (test)");
    }

    fn list(&self) -> Result<Vec<TrashItem>, PlatformError> {
        Ok(Vec::new())
    }

    fn restore(&self, _items: &[TrashItemId]) -> Result<(), PlatformError> {
        Ok(())
    }

    fn purge(&self, _items: &[TrashItemId]) -> Result<(), PlatformError> {
        Ok(())
    }
}
