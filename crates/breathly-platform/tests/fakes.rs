//! Les faux s'utilisent depuis un autre crate, par l'API publique seule, derrière les traits.

use std::error::Error;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::time::Duration;

use breathly_platform::fake::{FakeDisks, FakeProcesses, FakeSystem, MemoryTrash, SystemFacts};
use breathly_platform::{
    ByteSize, CpuTimes, DiskInspector, Executable, ExecutableOrigin, InvalidValue, MemoryStatus,
    MemoryUsage, Packaging, ProcessId, ProcessInspector, ProcessName, ProcessRole, ProcessSnapshot,
    SwapDevice, SwapKind, SwapStatus, SystemInspector, TrashProvider, Volume, VolumeRole,
};

type TestResult = Result<(), Box<dyn Error>>;

const GIB: u64 = 1 << 30;
const EIGHT_THREADS: NonZeroU16 = NonZeroU16::new(8).unwrap();

fn gib(count: u64) -> ByteSize {
    ByteSize::from_bytes(count * GIB)
}

fn editor(cpu_seconds: u64) -> Result<ProcessSnapshot, InvalidValue> {
    Ok(ProcessSnapshot {
        pid: ProcessId::new(42),
        name: ProcessName::try_from("code".to_owned())?,
        executable: Executable::Readable {
            path: PathBuf::from("/usr/share/code/code"),
            origin: ExecutableOrigin::Protected,
            packaging: Packaging::ProvidedByPackage,
        },
        role: ProcessRole::Regular,
        memory: MemoryUsage::Proportional(gib(1)),
        cpu_time: Duration::from_secs(cpu_seconds),
    })
}

fn cpu(busy: u64, total: u64) -> Result<CpuTimes, InvalidValue> {
    CpuTimes::new(
        Duration::from_secs(busy),
        Duration::from_secs(total),
        EIGHT_THREADS,
    )
}

#[test]
fn two_process_snapshots_give_the_cpu_used_during_the_window() -> TestResult {
    let inspector: Box<dyn ProcessInspector> = Box::new(FakeProcesses::snapshots(
        Ok(vec![editor(10)?]),
        [Ok(vec![editor(12)?])],
    ));
    let before = inspector.processes()?;
    let after = inspector.processes()?;
    assert_eq!(
        after[0].cpu_time.checked_sub(before[0].cpu_time),
        Some(Duration::from_secs(2))
    );
    assert_eq!(after[0].memory.size(), gib(1));
    Ok(())
}

#[test]
fn the_fake_system_reports_memory_zram_and_successive_cpu_times() -> TestResult {
    let zram = SwapDevice {
        kind: SwapKind::Zram,
        size: gib(4),
    };
    let system: Box<dyn SystemInspector> = Box::new(FakeSystem::new(SystemFacts {
        memory: Ok(MemoryStatus::new(gib(16), gib(2))?),
        swap: Ok(SwapStatus::new(gib(4), gib(1), vec![zram])?),
        cpu: vec![Ok(cpu(100, 1000)?), Ok(cpu(140, 1040)?)],
    }));
    assert_eq!(system.memory()?.available(), gib(2));
    assert_eq!(system.swap()?.devices()[0].kind, SwapKind::Zram);
    let (first, second) = (system.cpu_times()?, system.cpu_times()?);
    assert_eq!(
        second.busy().checked_sub(first.busy()),
        Some(Duration::from_secs(40))
    );
    Ok(())
}

#[test]
fn the_fake_disks_and_trash_work_behind_their_traits() -> TestResult {
    let space = (gib(1), ByteSize::from_bytes(1));
    let root = Volume::new(PathBuf::from("/"), VolumeRole::System, space)?;
    let disks: Box<dyn DiskInspector> = Box::new(FakeDisks::new(Ok(vec![root])));
    assert_eq!(disks.volumes()?[0].role(), VolumeRole::System);

    let memory_trash = MemoryTrash::new();
    memory_trash.add_file("/fake/quarantaine/a", ByteSize::from_bytes(5));
    let trash: &dyn TrashProvider = &memory_trash;
    let id = trash.send(Path::new("/fake/quarantaine/a"))?;
    assert_eq!(trash.list()?[0].id, id);
    Ok(())
}
