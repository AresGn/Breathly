use std::path::{Path, PathBuf};

use crate::error::PlatformError;
use crate::paths::{StandardPaths, UserFolder};

/// Dossiers standard rangés sous une racine choisie par le test (en général un dossier temporaire).
pub struct FixedStandardPaths {
    root: PathBuf,
}

impl FixedStandardPaths {
    #[must_use]
    pub fn under(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl StandardPaths for FixedStandardPaths {
    fn config_dir(&self) -> Result<PathBuf, PlatformError> {
        Ok(self.root.join("config"))
    }

    fn state_dir(&self) -> Result<PathBuf, PlatformError> {
        Ok(self.root.join("state"))
    }

    fn home_dir(&self) -> Result<PathBuf, PlatformError> {
        Ok(self.root.join("home"))
    }

    fn temp_dir(&self) -> Result<PathBuf, PlatformError> {
        Ok(self.root.join("tmp"))
    }

    fn user_folder(&self, folder: UserFolder) -> Result<PathBuf, PlatformError> {
        let name = match folder {
            UserFolder::Documents => "Documents",
            UserFolder::Pictures => "Pictures",
            UserFolder::Videos => "Videos",
            UserFolder::Music => "Music",
            UserFolder::Desktop => "Desktop",
        };
        Ok(self.root.join("home").join(name))
    }
}
