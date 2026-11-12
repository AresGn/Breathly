//! Composition des chemins avec des valeurs choisies : le vrai profil n'est jamais lu.

use std::collections::HashMap;

use super::*;

#[derive(Default)]
struct MapSources {
    vars: HashMap<&'static str, &'static str>,
    home: Option<&'static str>,
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
}

impl Sources for MapSources {
    fn var(&self, name: &str) -> Option<OsString> {
        self.vars.get(name).map(OsString::from)
    }

    fn home(&self) -> Option<PathBuf> {
        self.home.map(PathBuf::from)
    }

    fn user_folder(&self, folder: UserFolder) -> Option<PathBuf> {
        let name = match folder {
            UserFolder::Documents => "Documents",
            UserFolder::Pictures => "Pictures",
            UserFolder::Videos => "Videos",
            UserFolder::Music => "Music",
            UserFolder::Desktop => "Desktop",
        };
        self.home.map(|home| PathBuf::from(home).join(name))
    }
}

fn linux(sources: MapSources) -> ComposedPaths<MapSources> {
    ComposedPaths::new(PathStyle::Posix, sources)
}

fn windows(sources: MapSources) -> ComposedPaths<MapSources> {
    ComposedPaths::new(PathStyle::Windows, sources)
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

#[test]
fn linux_temp_is_tmpdir_or_slash_tmp_never_home() {
    let custom = linux(
        MapSources::default()
            .home("/home/awa")
            .with("TMPDIR", "/var/tmp"),
    );
    assert_eq!(custom.temp_dir(), Ok(PathBuf::from("/var/tmp")));
    let unset = linux(MapSources::default().home("/home/awa"));
    assert_eq!(unset.temp_dir(), Ok(PathBuf::from("/tmp")));
    let empty = linux(MapSources::default().home("/home/awa").with("TMPDIR", ""));
    assert_eq!(empty.temp_dir(), Ok(PathBuf::from("/tmp")));
}

#[test]
fn linux_temp_refuses_home_or_a_parent_of_home() {
    for tmpdir in ["/home/awa", "/home/awa/", "/home", "/"] {
        let paths = linux(
            MapSources::default()
                .home("/home/awa")
                .with("TMPDIR", tmpdir),
        );
        assert_eq!(
            paths.temp_dir(),
            Err(PlatformError::TempDirUnsafe),
            "{tmpdir}"
        );
    }
    let relative = linux(MapSources::default().with("TMPDIR", "tmp"));
    assert_eq!(relative.temp_dir(), Err(PlatformError::TempDirUnsafe));
}

#[test]
fn windows_temp_is_the_temp_variable() {
    let paths = windows(windows_session());
    assert_eq!(
        paths.temp_dir(),
        Ok(PathBuf::from(r"C:\Users\Awa\AppData\Local\Temp"))
    );
}

#[test]
fn windows_temp_missing_or_empty_is_refused_not_replaced_by_the_profile() {
    let without = windows(
        MapSources::default()
            .home(WIN_HOME)
            .with("USERPROFILE", WIN_HOME),
    );
    assert_eq!(without.temp_dir(), Err(PlatformError::TempDirUnsafe));
    let empty = windows(windows_session().with("TEMP", ""));
    assert_eq!(empty.temp_dir(), Err(PlatformError::TempDirUnsafe));
}

#[test]
fn windows_temp_equal_to_the_profile_or_its_parent_is_refused() {
    for temp in [
        r"C:\Users\Awa",
        r"C:\Users\Awa\",
        r"c:\users\AWA",
        r"C:/Users/Awa",
        r"C:\Users",
        r"C:\",
    ] {
        let paths = windows(windows_session().with("TEMP", temp));
        assert_eq!(
            paths.temp_dir(),
            Err(PlatformError::TempDirUnsafe),
            "{temp}"
        );
    }
}

#[test]
fn windows_temp_equal_to_userprofile_is_refused_even_without_a_known_home() {
    let paths = windows(
        MapSources::default()
            .with("USERPROFILE", WIN_HOME)
            .with("TEMP", WIN_HOME),
    );
    assert_eq!(paths.temp_dir(), Err(PlatformError::TempDirUnsafe));
}

#[test]
fn windows_temp_must_be_absolute() {
    for temp in ["Temp", r"..\Temp", r"\Temp"] {
        let paths = windows(windows_session().with("TEMP", temp));
        assert_eq!(
            paths.temp_dir(),
            Err(PlatformError::TempDirUnsafe),
            "{temp}"
        );
    }
}

#[test]
fn windows_temp_follows_temp_even_when_tmp_differs() {
    let paths = windows(windows_session().with("TMP", r"D:\Scratch"));
    assert_eq!(
        paths.temp_dir(),
        Ok(PathBuf::from(r"C:\Users\Awa\AppData\Local\Temp"))
    );
}
