//! Composition des chemins avec des valeurs choisies : le vrai profil n'est jamais lu.

use std::collections::HashMap;

use std::path::Path;

use super::*;

/// Sources choisies ; `links` simule un lien symbolique ou un nom court Windows, tout autre chemin existe tel quel.
#[derive(Default)]
struct MapSources {
    vars: HashMap<&'static str, &'static str>,
    home: Option<&'static str>,
    links: HashMap<&'static str, &'static str>,
}

impl MapSources {
    fn with(mut self, name: &'static str, value: &'static str) -> Self {
        self.vars.insert(name, value);
        self
    }

    fn home(mut self, home: &'static str) -> Self {
        self.home = Some(home);
        self
    }

    fn link(mut self, from: &'static str, to: &'static str) -> Self {
        self.links.insert(from, to);
        self
    }
}

impl Sources for MapSources {
    fn var(&self, name: &str) -> Option<OsString> {
        self.vars.get(name).map(OsString::from)
    }

    fn home(&self) -> Option<PathBuf> {
        self.home.map(PathBuf::from)
    }

    fn user_folder(&self, folder: UserFolder) -> Option<PathBuf> {
        // Pas de `Path::join` : sous Windows il insère `\`, et `PathStyle::Posix` ne découpe que sur `/`.
        self.home.map(|home| {
            let home = home.trim_end_matches(['/', '\\']);
            PathBuf::from(format!("{home}/{}", folder.english_name()))
        })
    }

    fn resolve(&self, path: &Path) -> Option<PathBuf> {
        let text = path.to_str()?;
        Some(PathBuf::from(self.links.get(text).copied().unwrap_or(text)))
    }
}

fn linux(sources: MapSources) -> SystemStandardPaths {
    SystemStandardPaths::with_sources(PathStyle::Posix, Box::new(sources))
}

fn windows(sources: MapSources) -> SystemStandardPaths {
    SystemStandardPaths::with_sources(PathStyle::Windows, Box::new(sources))
}

const WIN_HOME: &str = r"C:\Users\Awa";

fn windows_session() -> MapSources {
    MapSources::default()
        .home(WIN_HOME)
        .with("USERPROFILE", WIN_HOME)
        .with("APPDATA", r"C:\Users\Awa\AppData\Roaming")
        .with("LOCALAPPDATA", r"C:\Users\Awa\AppData\Local")
        .with("TEMP", r"C:\Users\Awa\AppData\Local\Temp")
}

#[test]
fn linux_uses_xdg_variables_when_absolute() {
    let paths = linux(
        MapSources::default()
            .home("/home/awa")
            .with("XDG_CONFIG_HOME", "/srv/config")
            .with("XDG_STATE_HOME", "/srv/state"),
    );
    assert_eq!(
        paths.config_dir(),
        Ok(PathBuf::from("/srv/config/breathly"))
    );
    assert_eq!(paths.state_dir(), Ok(PathBuf::from("/srv/state/breathly")));
}

#[test]
fn linux_falls_back_to_the_xdg_defaults_under_home() {
    let paths = linux(MapSources::default().home("/home/awa"));
    assert_eq!(
        paths.config_dir(),
        Ok(PathBuf::from("/home/awa/.config/breathly"))
    );
    assert_eq!(
        paths.state_dir(),
        Ok(PathBuf::from("/home/awa/.local/state/breathly"))
    );
}

#[test]
fn linux_ignores_empty_and_relative_xdg_variables() {
    let paths = linux(
        MapSources::default()
            .home("/home/awa")
            .with("XDG_CONFIG_HOME", "")
            .with("XDG_STATE_HOME", "relatif/state"),
    );
    assert_eq!(
        paths.config_dir(),
        Ok(PathBuf::from("/home/awa/.config/breathly"))
    );
    assert_eq!(
        paths.state_dir(),
        Ok(PathBuf::from("/home/awa/.local/state/breathly"))
    );
}

#[test]
fn linux_without_variable_nor_home_has_no_app_dir() {
    let paths = linux(MapSources::default());
    assert_eq!(paths.state_dir(), Err(unavailable(StandardPath::State)));
    assert_eq!(paths.home_dir(), Err(unavailable(StandardPath::Home)));
}

#[test]
fn windows_composes_roaming_and_local_app_data() {
    let paths = windows(windows_session());
    let roaming = PathBuf::from(r"C:\Users\Awa\AppData\Roaming").join("Breathly");
    let local = PathBuf::from(r"C:\Users\Awa\AppData\Local").join("Breathly");
    assert_eq!(paths.config_dir(), Ok(roaming));
    assert_eq!(paths.state_dir(), Ok(local));
}

#[test]
fn windows_does_not_guess_app_data_from_home() {
    let paths = windows(MapSources::default().home(WIN_HOME));
    assert_eq!(paths.config_dir(), Err(unavailable(StandardPath::Config)));
    assert_eq!(paths.state_dir(), Err(unavailable(StandardPath::State)));
}

#[test]
fn user_folders_come_from_the_system_and_must_be_absolute() {
    let paths = linux(MapSources::default().home("/home/awa"));
    assert_eq!(
        paths.user_folder(UserFolder::Music),
        Ok(PathBuf::from("/home/awa/Music"))
    );
    let relative = linux(MapSources::default().home("awa"));
    assert_eq!(
        relative.user_folder(UserFolder::Desktop),
        Err(unavailable(StandardPath::Folder(UserFolder::Desktop)))
    );
}

mod temp;
