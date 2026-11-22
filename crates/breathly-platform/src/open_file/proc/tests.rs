//! Lecture d'un faux `/proc` fabriqué en dossier temporaire : les descripteurs sont de vrais liens vers de vrais
//! fichiers d'essai, donc leur identité (dev, inode) est celle que le noyau donnerait.

use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, symlink};
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::{OpenFileInspector, OpenFileStatus, ProcFiles};
use crate::safe_path::CanonicalPath;
use crate::units::{ProcessId, ProcessName};

type TestResult = Result<(), Box<dyn Error>>;

const OWN: u32 = 100;

struct FakeProc {
    sandbox: TempDir,
}

impl FakeProc {
    fn new() -> Result<Self, Box<dyn Error>> {
        let sandbox = tempfile::tempdir()?;
        fs::create_dir_all(sandbox.path().join("proc").join(OWN.to_string()).join("fd"))?;
        Ok(Self { sandbox })
    }

    fn proc_dir(&self) -> PathBuf {
        self.sandbox.path().join("proc")
    }

    fn file(&self, name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let path = self.sandbox.path().join(name);
        fs::create_dir_all(path.parent().ok_or("chemin sans parent")?)?;
        fs::write(&path, b"x")?;
        Ok(path)
    }

    fn process(&self, pid: u32, name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let base = self.proc_dir().join(pid.to_string());
        fs::create_dir_all(base.join("fd"))?;
        fs::write(base.join("comm"), format!("{name}\n"))?;
        Ok(base)
    }

    fn hold(&self, holder: (u32, &str), target: &Path) -> TestResult {
        let base = self.process(holder.0, holder.1)?;
        symlink(target, base.join("fd").join("3"))?;
        Ok(())
    }

    fn status(&self, item: &Path) -> Result<OpenFileStatus, Box<dyn Error>> {
        let files = ProcFiles::under(self.proc_dir(), ProcessId::new(OWN));
        Ok(files.check(&CanonicalPath::resolve(item)?))
    }
}

fn programs(names: &[&str]) -> Result<OpenFileStatus, Box<dyn Error>> {
    let programs = names
        .iter()
        .map(|name| ProcessName::try_from((*name).to_owned()))
        .collect::<Result<_, _>>()?;
    Ok(OpenFileStatus::InUse { programs })
}

#[test]
fn a_file_held_by_a_descriptor_is_in_use_and_names_the_program() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("cache/a.bin")?;
    fake.hold((200, "editeur"), &file)?;
    assert_eq!(fake.status(&file)?, programs(&["editeur"])?);
    Ok(())
}

#[test]
fn a_file_nobody_holds_is_free() -> TestResult {
    let fake = FakeProc::new()?;
    let held = fake.file("cache/a.bin")?;
    let other = fake.file("cache/b.bin")?;
    fake.hold((200, "editeur"), &held)?;
    assert_eq!(fake.status(&other)?, OpenFileStatus::Free);
    Ok(())
}

#[test]
fn a_folder_is_in_use_when_a_file_below_it_is_held() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("cache/deep/a.bin")?;
    fake.hold((200, "editeur"), &file)?;
    assert_eq!(
        fake.status(&fake.sandbox.path().join("cache"))?,
        programs(&["editeur"])?
    );
    fake.file("elsewhere/b.bin")?;
    assert_eq!(
        fake.status(&fake.sandbox.path().join("elsewhere"))?,
        OpenFileStatus::Free
    );
    Ok(())
}

#[test]
fn the_programs_are_listed_once_and_sorted() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("a.bin")?;
    fake.hold((300, "zip"), &file)?;
    fake.hold((200, "alpha"), &file)?;
    fake.hold((201, "alpha"), &file)?;
    assert_eq!(fake.status(&file)?, programs(&["alpha", "zip"])?);
    Ok(())
}

#[test]
fn breathly_own_files_are_ignored() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("a.bin")?;
    fake.hold((OWN, "breathly"), &file)?;
    assert_eq!(fake.status(&file)?, OpenFileStatus::Free);
    Ok(())
}

#[test]
fn a_file_mapped_in_memory_without_a_descriptor_is_in_use() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("lib.so")?;
    let metadata = fs::metadata(&file)?;
    let base = fake.process(200, "player")?;
    let (major, minor) = (libc::major(metadata.dev()), libc::minor(metadata.dev()));
    let line = format!(
        "7f00-7f10 r--p 00000000 {major:02x}:{minor:02x} {} {}\n",
        metadata.ino(),
        file.display()
    );
    fs::write(base.join("maps"), line)?;
    assert_eq!(fake.status(&file)?, programs(&["player"])?);
    Ok(())
}

#[test]
fn a_deleted_file_still_mapped_keeps_its_folder_in_use() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("cache/gone.bin")?;
    let base = fake.process(200, "player")?;
    fs::write(
        base.join("maps"),
        format!("7f00-7f10 r--p 0 00:00 0 {} (deleted)\n", file.display()),
    )?;
    assert_eq!(
        fake.status(&fake.sandbox.path().join("cache"))?,
        programs(&["player"])?
    );
    Ok(())
}

#[test]
fn a_file_whose_name_is_not_utf8_is_still_seen_through_a_descriptor() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.sandbox.path().join(OsStr::from_bytes(b"caf\xe9.bin"));
    fs::write(&file, b"x")?;
    fake.hold((200, "editeur"), &file)?;
    assert_eq!(fake.status(&file)?, programs(&["editeur"])?);
    Ok(())
}

#[test]
fn a_file_whose_name_is_not_utf8_is_still_seen_through_a_mapping() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.sandbox.path().join(OsStr::from_bytes(b"caf\xe9.so"));
    fs::write(&file, b"x")?;
    let base = fake.process(200, "player")?;
    let mut line = b"7f00-7f10 r--p 0 00:00 0 ".to_vec();
    line.extend_from_slice(file.as_os_str().as_bytes());
    line.push(b'\n');
    fs::write(base.join("maps"), line)?;
    assert_eq!(fake.status(&file)?, programs(&["player"])?);
    Ok(())
}

#[test]
fn a_mapping_whose_device_differs_from_stat_is_matched_by_its_path() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("lib.so")?;
    let base = fake.process(200, "player")?;
    let line = format!("7f00-7f10 r--p 0 ff:ff 987654321 {}\n", file.display());
    fs::write(base.join("maps"), line)?;
    assert_eq!(fake.status(&file)?, programs(&["player"])?);
    Ok(())
}

#[test]
fn a_missing_proc_makes_the_check_unavailable() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let file = sandbox.path().join("a.bin");
    fs::write(&file, b"x")?;
    let files = ProcFiles::under(sandbox.path().join("no-proc"), ProcessId::new(OWN));
    assert_eq!(
        files.check(&CanonicalPath::resolve(&file)?),
        OpenFileStatus::Unavailable
    );
    Ok(())
}

#[test]
fn a_proc_without_even_the_own_descriptors_makes_the_check_unavailable() -> TestResult {
    let fake = FakeProc::new()?;
    let file = fake.file("a.bin")?;
    let files = ProcFiles::under(fake.proc_dir(), ProcessId::new(999));
    assert_eq!(
        files.check(&CanonicalPath::resolve(&file)?),
        OpenFileStatus::Unavailable
    );
    Ok(())
}
