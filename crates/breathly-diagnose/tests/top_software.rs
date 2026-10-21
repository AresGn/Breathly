//! Top 10 des logiciels (US-011), à partir de relevés du faux `ProcessInspector`.

use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use breathly_diagnose::{DiagnoseError, SamplingWindow, TOP_COUNT, top_software};
use breathly_platform::fake::FakeProcesses;
use breathly_platform::{
    ByteSize, Executable, ExecutableOrigin, InvalidValue, MemoryUsage, Packaging, ProcessId,
    ProcessInspector, ProcessName, ProcessRole, ProcessSnapshot,
};

type TestResult = Result<(), Box<dyn Error>>;

const MIB: u64 = 1 << 20;

/// Un processus : identifiant, nom, chemin de l'exécutable (None = illisible), Mo de mémoire, secondes de CPU.
struct Spec<'a>(u32, &'a str, Option<&'a str>, u64, u64);

fn process(spec: &Spec<'_>) -> Result<ProcessSnapshot, InvalidValue> {
    let Spec(pid, name, path, mib, cpu_seconds) = *spec;
    let executable = match path {
        Some(path) => Executable::Readable {
            path: PathBuf::from(path),
            origin: ExecutableOrigin::Protected,
            packaging: Packaging::ProvidedByPackage,
        },
        None => Executable::Unreadable,
    };
    Ok(ProcessSnapshot {
        pid: ProcessId::new(pid),
        name: ProcessName::try_from(name.to_owned())?,
        executable,
        role: ProcessRole::Regular,
        memory: MemoryUsage::Proportional(ByteSize::from_bytes(mib * MIB)),
        cpu_time: Duration::from_secs(cpu_seconds),
    })
}

fn snapshot(specs: &[Spec<'_>]) -> Result<Vec<ProcessSnapshot>, InvalidValue> {
    specs.iter().map(process).collect()
}

/// Deux relevés successifs, lus par le faux `ProcessInspector`.
fn two_snapshots(
    before: &[Spec<'_>],
    after: &[Spec<'_>],
) -> Result<(Vec<ProcessSnapshot>, Vec<ProcessSnapshot>), Box<dyn Error>> {
    let inspector = FakeProcesses::snapshots(Ok(snapshot(before)?), [Ok(snapshot(after)?)]);
    Ok((inspector.processes()?, inspector.processes()?))
}

fn five_seconds() -> Result<SamplingWindow, DiagnoseError> {
    SamplingWindow::try_from(Duration::from_secs(5))
}

#[test]
fn processes_of_one_executable_are_one_software() -> TestResult {
    let code = Some("/usr/share/code/code");
    let specs = [
        Spec(1, "code", code, 100, 0),
        Spec(2, "code", code, 200, 0),
        Spec(3, "code", code, 300, 0),
    ];
    let (before, after) = two_snapshots(&specs, &specs)?;
    let top = top_software(&before, &after, five_seconds()?);
    assert_eq!(top.by_memory.len(), 1);
    assert_eq!(top.by_memory[0].name.as_str(), "code");
    assert_eq!(top.by_memory[0].memory, ByteSize::from_bytes(600 * MIB));
    assert_eq!(top.by_memory[0].processes.len(), 3);
    assert_eq!(
        top.by_memory[0].processes[0].path,
        Some(PathBuf::from("/usr/share/code/code"))
    );
    Ok(())
}

#[test]
fn each_list_keeps_at_most_ten_software_sorted_by_decreasing_use() -> TestResult {
    let names: Vec<String> = (1..=12).map(|n| format!("logiciel{n:02}")).collect();
    let paths: Vec<String> = names
        .iter()
        .map(|name| format!("/usr/bin/{name}"))
        .collect();
    let before: Vec<Spec<'_>> = (0..12)
        .map(|i| Spec(i, &names[i as usize], Some(&paths[i as usize]), 0, 0))
        .collect();
    let after: Vec<Spec<'_>> = (0..12)
        .map(|i| {
            Spec(
                i,
                &names[i as usize],
                Some(&paths[i as usize]),
                u64::from(i) + 1,
                u64::from(i) + 1,
            )
        })
        .collect();
    let (before, after) = two_snapshots(&before, &after)?;
    let top = top_software(&before, &after, five_seconds()?);
    assert_eq!(
        (top.by_memory.len(), top.by_cpu.len()),
        (TOP_COUNT, TOP_COUNT)
    );
    assert_eq!(top.by_memory[0].name.as_str(), "logiciel12");
    assert_eq!(top.by_memory[9].name.as_str(), "logiciel03");
    assert_eq!(top.by_cpu[0].name.as_str(), "logiciel12");
    assert!(top.by_cpu.windows(2).all(|pair| pair[0].cpu >= pair[1].cpu));
    Ok(())
}

#[test]
fn cpu_is_the_share_of_one_core_used_between_the_two_snapshots() -> TestResult {
    let firefox = Some("/usr/lib/firefox/firefox");
    let before = [
        Spec(10, "firefox", firefox, 50, 100),
        Spec(11, "ancien", None, 10, 900),
    ];
    // 10 a consommé 5 s en 5 s ; 11 a été réutilisé par un autre programme ; 12 a démarré pendant la fenêtre.
    let after = [
        Spec(10, "firefox", firefox, 50, 105),
        Spec(11, "nouveau", None, 10, 2),
        Spec(12, "firefox", firefox, 50, 3),
    ];
    let (before, after) = two_snapshots(&before, &after)?;
    let top = top_software(&before, &after, five_seconds()?);
    let share = |name: &str| {
        top.by_cpu
            .iter()
            .find(|usage| usage.name.as_str() == name)
            .map(|usage| usage.cpu.get())
    };
    assert_eq!(share("firefox"), Some(1.6));
    assert_eq!(share("nouveau"), Some(0.4));
    Ok(())
}

#[test]
fn unreadable_executables_are_grouped_by_process_name_and_kernel_threads_are_left_out() -> TestResult
{
    let specs = [
        Spec(1, "sshd", None, 8, 0),
        Spec(2, "sshd", None, 8, 0),
        Spec(3, "kworker/0:1", None, 0, 0),
    ];
    let mut after = snapshot(&specs)?;
    after[2].role = ProcessRole::KernelThread;
    after[2].memory = MemoryUsage::Resident(ByteSize::from_bytes(MIB));
    let top = top_software(&snapshot(&specs)?, &after, five_seconds()?);
    assert_eq!(top.by_memory.len(), 1);
    assert_eq!(top.by_memory[0].name.as_str(), "sshd");
    assert_eq!(top.by_memory[0].processes[0].path, None);
    Ok(())
}

#[test]
fn software_without_cpu_use_is_not_in_the_cpu_list() -> TestResult {
    let specs = [Spec(1, "idle", Some("/usr/bin/idle"), 10, 7)];
    let (before, after) = two_snapshots(&specs, &specs)?;
    let top = top_software(&before, &after, five_seconds()?);
    assert_eq!((top.by_memory.len(), top.by_cpu.len()), (1, 0));
    Ok(())
}

#[test]
fn an_empty_sampling_window_is_refused() {
    assert_eq!(
        SamplingWindow::try_from(Duration::ZERO),
        Err(DiagnoseError::EmptySamplingWindow)
    );
}
