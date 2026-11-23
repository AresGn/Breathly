//! Corbeille de test : le faux en mémoire ne voit pas le disque, ce pont le relie sans toucher la vraie corbeille.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use breathly_platform::fake::MemoryTrash;
use breathly_platform::{ByteSize, PlatformError, TrashItem, TrashItemId, TrashProvider};

pub struct BridgedTrash<'a> {
    inner: &'a MemoryTrash,
}

impl<'a> BridgedTrash<'a> {
    pub const fn new(inner: &'a MemoryTrash) -> Self {
        Self { inner }
    }
}

impl TrashProvider for BridgedTrash<'_> {
    fn send(&self, path: &Path) -> Result<TrashItemId, PlatformError> {
        remember(self.inner, path)?;
        let id = self.inner.send(path)?;
        remove_real(path)?;
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

pub struct Unavailable;

impl TrashProvider for Unavailable {
    fn send(&self, _path: &Path) -> Result<TrashItemId, PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }

    fn list(&self) -> Result<Vec<TrashItem>, PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }

    fn restore(&self, _items: &[TrashItemId]) -> Result<(), PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }

    fn purge(&self, _items: &[TrashItemId]) -> Result<(), PlatformError> {
        Err(PlatformError::TrashUnavailable)
    }
}

pub struct StopAfterFirst<'a> {
    inner: BridgedTrash<'a>,
    stop: &'a breathly_disk::StopFlag,
    seen: AtomicUsize,
}

impl<'a> StopAfterFirst<'a> {
    pub fn new(inner: BridgedTrash<'a>, stop: &'a breathly_disk::StopFlag) -> Self {
        Self {
            inner,
            stop,
            seen: AtomicUsize::new(0),
        }
    }
}

impl TrashProvider for StopAfterFirst<'_> {
    fn send(&self, path: &Path) -> Result<TrashItemId, PlatformError> {
        let id = self.inner.send(path)?;
        if self.seen.fetch_add(1, Ordering::Relaxed) == 0 {
            self.stop.request();
        }
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

fn remember(trash: &MemoryTrash, path: &Path) -> Result<(), PlatformError> {
    let meta = fs::symlink_metadata(path).map_err(|_| PlatformError::TrashSourceMissing)?;
    trash.add_file(path, ByteSize::from_bytes(meta.len()));
    Ok(())
}

fn remove_real(path: &Path) -> Result<(), PlatformError> {
    let meta = fs::symlink_metadata(path).map_err(|_| PlatformError::TrashUnavailable)?;
    let removed = if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    removed.map_err(|_| PlatformError::TrashUnavailable)
}
