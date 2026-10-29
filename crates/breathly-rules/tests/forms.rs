//! Les quatre formes d'une règle et les chemins autorisés (US-060.2, S5, TS-09).

mod common;

use breathly_rules::{Field, Os, PathRefusal, RuleDefect, Target, parse_cleanup_rules};
use common::{Change, defect_of, rule_file};

const LINUX_ONLY: [Change<'static>; 2] = [("os", Some(r#"["linux"]"#)), ("paths.windows", None)];

fn linux_rule(changes: &[Change<'_>]) -> String {
    rule_file(&[LINUX_ONLY.as_slice(), changes].concat())
}

fn refusal_of_linux_path(path: &str) -> PathRefusal {
    let paths = format!("[{path:?}]");
    let defect = defect_of(
        parse_cleanup_rules(&linux_rule(&[("paths.linux", Some(&paths))])),
        "npm-cache",
    );
    let RuleDefect::InvalidPath {
        os: Os::Linux,
        path: refused,
        refusal,
    } = defect
    else {
        panic!("chemin « {path} » : refus de chemin attendu, obtenu {defect:?}");
    };
    assert_eq!(refused, path);
    refusal
}

#[test]
fn paths_outside_the_allowed_roots_are_refused() {
    for path in [
        "/",
        "/home/someone/.cache",
        "C:/Users",
        "$HOME/.cache",
        ".cache",
        "~someone/.cache",
    ] {
        assert_eq!(
            refusal_of_linux_path(path),
            PathRefusal::OutsideAllowedRoots,
            "{path}"
        );
    }
}

#[test]
fn a_root_alone_is_refused_for_a_cleaned_folder() {
    assert_eq!(refusal_of_linux_path("~"), PathRefusal::RootItself);
}

#[test]
fn protected_user_folders_are_refused_in_both_languages_and_any_case() {
    for path in [
        "~/Documents/cache",
        "~/Images",
        "~/Vidéos",
        "~/music",
        "~/Bureau/x",
        "~/DESKTOP",
    ] {
        assert_eq!(
            refusal_of_linux_path(path),
            PathRefusal::ProtectedFolder,
            "{path}"
        );
    }
}

#[test]
fn git_folders_parent_components_and_patterns_are_refused() {
    assert_eq!(
        refusal_of_linux_path("~/projet/.git"),
        PathRefusal::GitFolder
    );
    assert_eq!(
        refusal_of_linux_path("~/.cache/../Documents"),
        PathRefusal::NotNormalized
    );
    assert_eq!(
        refusal_of_linux_path("~/.cache//x"),
        PathRefusal::NotNormalized
    );
    assert_eq!(
        refusal_of_linux_path("~/.cache/*"),
        PathRefusal::ForbiddenCharacter
    );
    assert_eq!(
        refusal_of_linux_path(r"~\.cache"),
        PathRefusal::OutsideAllowedRoots
    );
    assert_eq!(
        refusal_of_linux_path(r"~/.cache\x"),
        PathRefusal::ForbiddenCharacter
    );
}

#[test]
fn a_windows_root_under_linux_is_refused() {
    assert_eq!(
        refusal_of_linux_path("%TEMP%/x"),
        PathRefusal::RootOfOtherOs
    );
}

#[test]
fn windows_protected_folders_and_drive_paths_are_refused() {
    let file = rule_file(&[("paths.windows", Some(r#"["%USERPROFILE%/Pictures/cache"]"#))]);
    let defect = defect_of(parse_cleanup_rules(&file), "npm-cache");
    assert!(matches!(
        defect,
        RuleDefect::InvalidPath {
            refusal: PathRefusal::ProtectedFolder,
            ..
        }
    ));
    let file = rule_file(&[("paths.windows", Some(r#"["%LOCALAPPDATA%/C:/x"]"#))]);
    let defect = defect_of(parse_cleanup_rules(&file), "npm-cache");
    assert!(matches!(
        defect,
        RuleDefect::InvalidPath {
            refusal: PathRefusal::ForbiddenCharacter,
            ..
        }
    ));
}

#[test]
fn every_listed_os_needs_paths_and_no_other_os_may_have_some() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("paths.windows", None)])),
        "npm-cache",
    );
    assert_eq!(defect, RuleDefect::MissingPathsForOs { os: Os::Windows });
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("os", Some(r#"["linux"]"#))])),
        "npm-cache",
    );
    assert_eq!(defect, RuleDefect::PathsForUnlistedOs { os: Os::Windows });
}

#[test]
fn a_marked_folder_rule_may_search_the_whole_home_folder() {
    let rules = parse_cleanup_rules(&linux_rule(&[
        ("kind", Some(r#""marked-folder""#)),
        ("risk", Some(r#""review""#)),
        ("folder", Some(r#""node_modules""#)),
        ("marker", Some(r#""package.json""#)),
        ("paths.linux", Some(r#"["~"]"#)),
    ]))
    .unwrap();

    let Target::MarkedFolder {
        folder,
        marker,
        roots,
    } = rules[0].target()
    else {
        panic!("forme « marked-folder » attendue")
    };
    assert_eq!(
        (folder.as_str(), marker.as_str()),
        ("node_modules", "package.json")
    );
    assert!(roots[&Os::Linux][0].components().is_empty());
}

#[test]
fn a_marked_folder_rule_needs_a_simple_folder_name_and_a_marker() {
    let base = [
        ("kind", Some(r#""marked-folder""#)),
        ("risk", Some(r#""review""#)),
        ("folder", Some(r#""target""#)),
    ];
    let defect = defect_of(parse_cleanup_rules(&linux_rule(&base)), "npm-cache");
    assert_eq!(
        defect,
        RuleDefect::MissingField {
            field: Field::Marker
        }
    );
    let file = linux_rule(&[base.as_slice(), &[("marker", Some(r#""../Cargo.toml""#))]].concat());
    let defect = defect_of(parse_cleanup_rules(&file), "npm-cache");
    assert!(
        matches!(
            defect,
            RuleDefect::InvalidName {
                field: Field::Marker,
                ..
            }
        ),
        "{defect:?}"
    );
}

#[test]
fn a_files_by_extension_rule_lists_plain_extensions() {
    let installers = [
        ("kind", Some(r#""files-by-extension""#)),
        ("risk", Some(r#""review""#)),
        ("extensions", Some(r#"["deb", "appimage"]"#)),
        (
            "paths.linux",
            Some(r#"["~/Téléchargements", "~/Downloads"]"#),
        ),
    ];
    let rules = parse_cleanup_rules(&linux_rule(&installers)).unwrap();
    assert!(
        matches!(rules[0].target(), Target::FilesByExtension { extensions, .. } if extensions.len() == 2)
    );

    for refused in [
        r#"[".deb"]"#,
        r#"["kdbx"]"#,
        r#"["gpg"]"#,
        r#"["DEB"]"#,
        "[]",
    ] {
        let file = linux_rule(&[installers.as_slice(), &[("extensions", Some(refused))]].concat());
        let defect = defect_of(parse_cleanup_rules(&file), "npm-cache");
        assert!(
            matches!(
                defect,
                RuleDefect::InvalidExtension { .. } | RuleDefect::EmptyValue { .. }
            ),
            "{refused} : {defect:?}"
        );
    }
}

#[test]
fn the_system_trash_rule_has_no_path() {
    let trash = [
        ("kind", Some(r#""system-trash""#)),
        ("risk", Some(r#""review""#)),
        ("paths.linux", None),
        ("paths.windows", None),
    ];
    let rules = parse_cleanup_rules(&rule_file(&trash)).unwrap();
    assert_eq!(rules[0].target(), &Target::SystemTrash);
}

#[test]
fn a_system_trash_rule_with_a_path_is_refused() {
    let file = rule_file(&[
        ("kind", Some(r#""system-trash""#)),
        ("risk", Some(r#""review""#)),
        ("paths.windows", None),
    ]);
    let defect = defect_of(parse_cleanup_rules(&file), "npm-cache");
    assert_eq!(defect, RuleDefect::PathsForSystemTrash);
}

#[test]
fn a_field_of_another_form_is_refused_rather_than_ignored() {
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("marker", Some(r#""package.json""#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::FieldNotAllowedForKind {
            field: Field::Marker
        }
    );
    let defect = defect_of(
        parse_cleanup_rules(&rule_file(&[("kind", Some(r#""cache""#))])),
        "npm-cache",
    );
    assert_eq!(
        defect,
        RuleDefect::UnknownKind {
            value: "cache".to_owned()
        }
    );
}
