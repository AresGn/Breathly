//! Top 10 des logiciels par RAM et par CPU, processus regroupés par logiciel (D3, US-011).
//!
//! Breathly ne ferme aucun logiciel en V0.1 (US-011.3) : ces données n'ont ni action ni commande de fermeture.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use breathly_platform::{
    ByteSize, Executable, ProcessId, ProcessName, ProcessRole, ProcessSnapshot,
};

use crate::error::DiagnoseError;

/// Nombre de logiciels par liste (US-011.2).
pub const TOP_COUNT: usize = 10;

/// Durée entre les deux relevés de processus ; jamais nulle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingWindow(Duration);

impl TryFrom<Duration> for SamplingWindow {
    type Error = DiagnoseError;

    fn try_from(raw: Duration) -> Result<Self, Self::Error> {
        if raw.is_zero() {
            return Err(DiagnoseError::EmptySamplingWindow);
        }
        Ok(Self(raw))
    }
}

/// Nom d'un logiciel : nom de son exécutable, ou du processus si l'exécutable est illisible.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SoftwareName(String);

impl SoftwareName {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Part d'un cœur occupée pendant la fenêtre : 1,0 = un cœur entier.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CoreShare(f64);

impl CoreShare {
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// Un processus d'un logiciel, pour le mode avancé (noms bruts et chemins).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessDetail {
    pub pid: ProcessId,
    pub name: ProcessName,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoftwareUsage {
    pub name: SoftwareName,
    pub memory: ByteSize,
    pub cpu: CoreShare,
    pub processes: Vec<ProcessDetail>,
}

/// Deux listes de `TOP_COUNT` logiciels au plus, triées par consommation décroissante.
#[derive(Debug, Clone, PartialEq)]
pub struct TopSoftware {
    pub by_memory: Vec<SoftwareUsage>,
    pub by_cpu: Vec<SoftwareUsage>,
}

fn software_name(process: &ProcessSnapshot) -> SoftwareName {
    let executable_name = match &process.executable {
        Executable::Readable { path, .. } => path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
        Executable::Unreadable => None,
    };
    SoftwareName(executable_name.unwrap_or_else(|| process.name.as_str().to_owned()))
}

/// Temps de processeur consommé pendant la fenêtre. Un processus absent du premier relevé, ou dont l'identifiant a
/// été réutilisé par un autre programme, a démarré pendant la fenêtre : tout son temps compte.
fn cpu_during_window(
    process: &ProcessSnapshot,
    before: &BTreeMap<ProcessId, &ProcessSnapshot>,
) -> Duration {
    before
        .get(&process.pid)
        .filter(|earlier| earlier.name == process.name)
        .and_then(|earlier| process.cpu_time.checked_sub(earlier.cpu_time))
        .unwrap_or(process.cpu_time)
}

fn detail(process: &ProcessSnapshot) -> ProcessDetail {
    let path = match &process.executable {
        Executable::Readable { path, .. } => Some(path.clone()),
        Executable::Unreadable => None,
    };
    ProcessDetail {
        pid: process.pid,
        name: process.name.clone(),
        path,
    }
}

fn is_counted(process: &ProcessSnapshot) -> bool {
    match process.role {
        ProcessRole::KernelThread => false,
        ProcessRole::SystemService | ProcessRole::Regular => true,
    }
}

/// Regroupe le second relevé par logiciel ; le CPU est la différence avec le premier relevé.
fn group(
    before: &[ProcessSnapshot],
    after: &[ProcessSnapshot],
    window: SamplingWindow,
) -> Vec<SoftwareUsage> {
    let earlier: BTreeMap<ProcessId, &ProcessSnapshot> = before
        .iter()
        .map(|process| (process.pid, process))
        .collect();
    let mut groups: BTreeMap<SoftwareName, (ByteSize, Duration, Vec<ProcessDetail>)> =
        BTreeMap::new();
    for process in after.iter().filter(|process| is_counted(process)) {
        let entry = groups.entry(software_name(process)).or_default();
        entry.0 = entry.0.saturating_add(process.memory.size());
        entry.1 = entry.1.saturating_add(cpu_during_window(process, &earlier));
        entry.2.push(detail(process));
    }
    groups
        .into_iter()
        .map(|(name, (memory, cpu, processes))| SoftwareUsage {
            name,
            memory,
            cpu: CoreShare(cpu.as_secs_f64() / window.0.as_secs_f64()),
            processes,
        })
        .collect()
}

/// Les `TOP_COUNT` premiers selon `key`, sans les logiciels à zéro ; à égalité, par nom (ordre stable).
fn top_by<K: PartialOrd>(
    software: &[SoftwareUsage],
    key: impl Fn(&SoftwareUsage) -> K,
    zero: &K,
) -> Vec<SoftwareUsage> {
    let mut ranked: Vec<&SoftwareUsage> =
        software.iter().filter(|usage| key(usage) > *zero).collect();
    ranked.sort_by(|a, b| {
        key(b)
            .partial_cmp(&key(a))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
    ranked.into_iter().take(TOP_COUNT).cloned().collect()
}

/// Top 10 des logiciels par RAM (second relevé) et par CPU (entre les deux relevés).
#[must_use]
pub fn top_software(
    before: &[ProcessSnapshot],
    after: &[ProcessSnapshot],
    window: SamplingWindow,
) -> TopSoftware {
    let software = group(before, after, window);
    TopSoftware {
        by_memory: top_by(&software, |usage| usage.memory, &ByteSize::default()),
        by_cpu: top_by(&software, |usage| usage.cpu, &CoreShare(0.0)),
    }
}
