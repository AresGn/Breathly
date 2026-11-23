//! Erreurs des commandes : un code et une clé, jamais un chemin (SEC9).

use std::fmt;

use breathly_disk::CleanError;
use serde::Serialize;
use serde::ser::SerializeStruct;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopError {
    Clean(CleanError),
    AlreadyRunning,
    UnknownTask,
    ThreadFailed,
}

impl DesktopError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Clean(error) => error.code(),
            Self::AlreadyRunning => "desktop.cleanup_already_running",
            Self::UnknownTask => "desktop.unknown_task",
            Self::ThreadFailed => "desktop.thread_failed",
        }
    }

    #[must_use]
    pub const fn translation_key(self) -> &'static str {
        match self {
            Self::Clean(error) => error.translation_key(),
            Self::AlreadyRunning => "errors.desktop.already_running",
            Self::UnknownTask => "errors.desktop.unknown_task",
            Self::ThreadFailed => "errors.desktop.thread_failed",
        }
    }
}

impl From<CleanError> for DesktopError {
    fn from(error: CleanError) -> Self {
        Self::Clean(error)
    }
}

impl fmt::Display for DesktopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for DesktopError {}

impl Serialize for DesktopError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("DesktopError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("translationKey", self.translation_key())?;
        state.end()
    }
}
