//! `temp_dir` : choix de la variable, puis garde sur le chemin écrit et résolu.

use super::*;

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
    for ignored in ["", "tmp", "./tmp"] {
        let paths = linux(
            MapSources::default()
                .home("/home/awa")
                .with("TMPDIR", ignored),
        );
        assert_eq!(paths.temp_dir(), Ok(PathBuf::from("/tmp")), "{ignored}");
    }
}

#[test]
fn linux_unsafe_tmpdir_is_refused_not_replaced() {
    for tmpdir in ["/home/awa", "/home/awa/..", "/", "/home/awa/Documents/tmp"] {
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
}

#[test]
fn temp_is_refused_when_the_profile_is_unknown() {
    let posix = linux(MapSources::default().with("TMPDIR", "/var/tmp"));
    assert_eq!(posix.temp_dir(), Err(PlatformError::TempDirUnsafe));
    let windows = windows(MapSources::default().with("TEMP", r"D:\Temp"));
    assert_eq!(windows.temp_dir(), Err(PlatformError::TempDirUnsafe));
}

#[test]
fn temp_resolving_to_the_profile_is_refused() {
    let symlink = linux(
        MapSources::default()
            .home("/home/awa")
            .with("TMPDIR", "/var/tmp/awa")
            .link("/var/tmp/awa", "/home/awa"),
    );
    assert_eq!(symlink.temp_dir(), Err(PlatformError::TempDirUnsafe));
    let short_name = windows(
        windows_session()
            .with("TEMP", r"C:\Users\AWA~1")
            .link(r"C:\Users\AWA~1", r"\\?\C:\Users\Awa"),
    );
    assert_eq!(short_name.temp_dir(), Err(PlatformError::TempDirUnsafe));
}

#[test]
fn temp_resolving_inside_the_profile_is_accepted() {
    let short_temp = r"C:\Users\AWA~1\AppData\Local\Temp";
    let paths = windows(
        windows_session()
            .with("TEMP", short_temp)
            .link(short_temp, r"\\?\C:\Users\Awa\AppData\Local\Temp"),
    );
    assert_eq!(paths.temp_dir(), Ok(PathBuf::from(short_temp)));
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
    let empty = windows(windows_session().with("TEMP", "").with("TMP", ""));
    assert_eq!(empty.temp_dir(), Err(PlatformError::TempDirUnsafe));
}

#[test]
fn windows_temp_falls_back_to_tmp_only_when_temp_is_missing() {
    let missing = windows(
        windows_session()
            .with("TEMP", "")
            .with("TMP", r"D:\Scratch"),
    );
    assert_eq!(missing.temp_dir(), Ok(PathBuf::from(r"D:\Scratch")));
    let refused = windows(
        windows_session()
            .with("TEMP", WIN_HOME)
            .with("TMP", r"D:\Scratch"),
    );
    assert_eq!(refused.temp_dir(), Err(PlatformError::TempDirUnsafe));
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
fn windows_temp_follows_temp_even_when_tmp_differs() {
    let paths = windows(windows_session().with("TMP", r"D:\Scratch"));
    assert_eq!(
        paths.temp_dir(),
        Ok(PathBuf::from(r"C:\Users\Awa\AppData\Local\Temp"))
    );
}
