//! Chemins canoniques et ancrage (ADR 0018, ADR 0019), dans un dossier temporaire : jamais le vrai disque.

use std::error::Error;
use std::fs;
use std::path::Path;

use breathly_platform::{AnchorRefusal, CanonicalPath, PathAnchor};

type TestResult = Result<(), Box<dyn Error>>;

fn canonical(path: &Path) -> Result<CanonicalPath, Box<dyn Error>> {
    Ok(CanonicalPath::resolve(path)?)
}

#[test]
fn a_path_is_within_its_root_and_a_sibling_with_the_same_prefix_is_not() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache/inner"))?;
    fs::create_dir_all(sandbox.path().join("cache-other"))?;
    let root = canonical(&sandbox.path().join("cache"))?;
    assert!(canonical(&sandbox.path().join("cache/inner"))?.is_within(&root));
    assert!(root.is_within(&root));
    assert!(!canonical(&sandbox.path().join("cache-other"))?.is_within(&root));
    assert!(!root.is_within(&canonical(&sandbox.path().join("cache/inner"))?));
    Ok(())
}

#[test]
fn equality_follows_the_same_rule_as_is_within() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("a"))?;
    let first = canonical(&sandbox.path().join("a"))?;
    assert_eq!(first, canonical(&sandbox.path().join("a"))?);
    assert_ne!(first, canonical(sandbox.path())?);
    Ok(())
}

#[test]
fn a_missing_path_cannot_be_resolved() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    assert!(CanonicalPath::resolve(&sandbox.path().join("absent")).is_err());
    Ok(())
}

#[test]
fn the_debug_of_paths_and_anchors_never_shows_the_path() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("secret-root/item"))?;
    let root = canonical(&sandbox.path().join("secret-root"))?;
    let item = canonical(&sandbox.path().join("secret-root/item"))?;
    let anchor = PathAnchor::open(&root, &item)?;
    assert!(!format!("{root:?}{anchor:?}").contains("secret"));
    Ok(())
}

#[test]
fn an_anchor_keeps_the_name_of_its_item() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache/a/b"))?;
    fs::write(sandbox.path().join("cache/a/b/file.bin"), b"x")?;
    let root = canonical(&sandbox.path().join("cache"))?;
    let item = canonical(&sandbox.path().join("cache/a/b/file.bin"))?;
    assert_eq!(PathAnchor::open(&root, &item)?.file_name(), "file.bin");
    Ok(())
}

#[test]
fn the_root_itself_and_a_path_outside_are_refused() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache"))?;
    fs::create_dir_all(sandbox.path().join("elsewhere"))?;
    let root = canonical(&sandbox.path().join("cache"))?;
    let outside = canonical(&sandbox.path().join("elsewhere"))?;
    assert_eq!(
        PathAnchor::open(&root, &root).unwrap_err(),
        AnchorRefusal::PathChanged
    );
    assert_eq!(
        PathAnchor::open(&root, &outside).unwrap_err(),
        AnchorRefusal::PathChanged
    );
    Ok(())
}

#[test]
fn an_item_that_disappeared_after_resolution_is_refused() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache/a"))?;
    fs::write(sandbox.path().join("cache/a/file.bin"), b"x")?;
    let root = canonical(&sandbox.path().join("cache"))?;
    let item = canonical(&sandbox.path().join("cache/a/file.bin"))?;
    fs::remove_file(sandbox.path().join("cache/a/file.bin"))?;
    assert_eq!(
        PathAnchor::open(&root, &item).unwrap_err(),
        AnchorRefusal::PathChanged
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_folder_of_the_chain_turned_into_a_link_after_resolution_is_refused() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache/a"))?;
    fs::write(sandbox.path().join("cache/a/file.bin"), b"x")?;
    fs::create_dir_all(sandbox.path().join("protected"))?;
    fs::write(sandbox.path().join("protected/file.bin"), b"precious")?;
    let root = canonical(&sandbox.path().join("cache"))?;
    let item = canonical(&sandbox.path().join("cache/a/file.bin"))?;
    fs::rename(sandbox.path().join("cache/a"), sandbox.path().join("moved"))?;
    std::os::unix::fs::symlink(
        sandbox.path().join("protected"),
        sandbox.path().join("cache/a"),
    )?;
    assert_eq!(
        PathAnchor::open(&root, &item).unwrap_err(),
        AnchorRefusal::PathChanged
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_folder_without_permission_is_denied_not_lost() -> TestResult {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache/a"))?;
    fs::write(sandbox.path().join("cache/a/file.bin"), b"x")?;
    let root = canonical(&sandbox.path().join("cache"))?;
    let item = canonical(&sandbox.path().join("cache/a/file.bin"))?;
    let locked = sandbox.path().join("cache/a");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))?;
    let outcome = PathAnchor::open(&root, &item);
    let still_readable = fs::read_dir(&locked).is_ok();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o700))?;
    if still_readable {
        return Ok(());
    }
    assert_eq!(outcome.unwrap_err(), AnchorRefusal::PathDenied);
    Ok(())
}

#[cfg(windows)]
#[test]
fn a_folder_held_without_sharing_by_another_program_is_busy_not_lost() -> TestResult {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("cache/a"))?;
    fs::write(sandbox.path().join("cache/a/file.bin"), b"x")?;
    let root = canonical(&sandbox.path().join("cache"))?;
    let item = canonical(&sandbox.path().join("cache/a/file.bin"))?;
    let _exclusive = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(sandbox.path().join("cache/a"))?;
    assert_eq!(
        PathAnchor::open(&root, &item).unwrap_err(),
        AnchorRefusal::PathBusy
    );
    Ok(())
}

#[cfg(windows)]
#[test]
fn the_comparison_ignores_the_case_on_windows() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::create_dir_all(sandbox.path().join("Cache/Inner"))?;
    let root = canonical(&sandbox.path().join("Cache"))?;
    let item = canonical(&sandbox.path().join("cache/inner"))?;
    assert!(item.is_within(&root));
    Ok(())
}
