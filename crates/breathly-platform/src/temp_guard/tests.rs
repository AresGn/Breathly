//! Garde du dossier temporaire, jugé sur des chemins écrits : le vrai disque n'est jamais lu.

use super::*;

fn protected(profile: &str, folders: &[&str]) -> Protected {
    Protected {
        profiles: vec![PathBuf::from(profile)],
        folders: folders.iter().map(PathBuf::from).collect(),
    }
}

fn linux_home() -> Protected {
    protected("/home/awa", &["/home/awa/Documents", "/home/awa/Bureau"])
}

fn windows_home() -> Protected {
    protected(r"C:\Users\Awa", &[r"C:\Users\Awa\Documents", r"D:\Photos"])
}

fn assert_refused(style: PathStyle, home: &Protected, temps: &[&str]) {
    for temp in temps {
        assert!(!is_safe_temp(Path::new(temp), home, style), "{temp}");
    }
}

#[test]
fn ordinary_temp_folders_are_accepted() {
    assert!(is_safe_temp(
        Path::new("/tmp"),
        &linux_home(),
        PathStyle::Posix
    ));
    assert!(is_safe_temp(
        Path::new("/home/awa/.cache/tmp"),
        &linux_home(),
        PathStyle::Posix
    ));
    for temp in [
        r"C:\Users\Awa\AppData\Local\Temp",
        r"\\?\C:\Users\Awa\AppData\Local\Temp",
        r"D:\Temp",
    ] {
        assert!(
            is_safe_temp(Path::new(temp), &windows_home(), PathStyle::Windows),
            "{temp}"
        );
    }
}

#[test]
fn linux_temp_holding_or_equal_to_home_is_refused() {
    assert_refused(
        PathStyle::Posix,
        &linux_home(),
        &["/home/awa", "/home/awa/", "/home", "/"],
    );
}

#[test]
fn dot_and_dot_dot_are_refused_not_normalised() {
    assert_refused(
        PathStyle::Posix,
        &linux_home(),
        &["/home/awa/..", "/tmp/../home/awa", "/tmp/.", "/var/./tmp"],
    );
    assert_refused(
        PathStyle::Windows,
        &windows_home(),
        &[
            r"C:\Users\Awa\..",
            r"C:\Temp\..\Users\Awa",
            r"C:\Users\Awa\...",
            r"C:\Temp\.",
        ],
    );
}

#[test]
fn temp_in_or_above_a_user_folder_is_refused() {
    assert_refused(
        PathStyle::Posix,
        &linux_home(),
        &[
            "/home/awa/Documents",
            "/home/awa/Documents/tmp",
            "/home/awa/Bureau/x",
        ],
    );
    assert_refused(
        PathStyle::Windows,
        &windows_home(),
        &[r"C:\Users\Awa\Documents\Temp", r"d:\photos", r"D:\"],
    );
}

#[test]
fn windows_temp_holding_or_equal_to_the_profile_is_refused() {
    assert_refused(
        PathStyle::Windows,
        &windows_home(),
        &[
            r"C:\Users\Awa",
            r"C:\Users\Awa\",
            r"c:\users\AWA",
            r"C:/Users/Awa",
            r"C:\Users\Awa.",
            r"C:\Users\Awa. ",
            r"C:\Users",
            r"C:\",
        ],
    );
}

#[test]
fn windows_verbatim_prefixes_are_read_and_device_paths_refused() {
    assert_refused(
        PathStyle::Windows,
        &windows_home(),
        &[
            r"\\?\C:\Users\Awa",
            r"//?/C:/Users/Awa",
            r"\\?\c:\users",
            r"\\.\C:\Temp",
            r"\\?\GLOBALROOT\Device\HarddiskVolume1",
            r"C:\Temp:stream",
        ],
    );
    let unc = protected(r"\\serveur\profils\awa", &[]);
    assert_refused(
        PathStyle::Windows,
        &unc,
        &[r"\\?\UNC\serveur\profils\awa", r"\\serveur\profils"],
    );
    assert!(is_safe_temp(
        Path::new(r"\\serveur\temp"),
        &unc,
        PathStyle::Windows
    ));
}

#[test]
fn relative_temp_is_refused() {
    assert_refused(
        PathStyle::Windows,
        &windows_home(),
        &["Temp", r"..\Temp", r"\Temp"],
    );
    assert_refused(PathStyle::Posix, &linux_home(), &["tmp", "./tmp"]);
}

#[test]
fn unknown_or_unreadable_profile_refuses_everything() {
    let unknown = Protected::default();
    assert!(!is_safe_temp(Path::new("/tmp"), &unknown, PathStyle::Posix));
    let relative = protected("awa", &[]);
    assert!(!is_safe_temp(
        Path::new("/tmp"),
        &relative,
        PathStyle::Posix
    ));
}
