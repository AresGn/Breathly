//! Système sans détection de fichier ouvert : tout est refusé par prudence (ADR 0020).

use super::OpenFileStatus;
use crate::safe_path::CanonicalPath;

pub(super) const fn check(_item: &CanonicalPath) -> OpenFileStatus {
    OpenFileStatus::Unavailable
}
