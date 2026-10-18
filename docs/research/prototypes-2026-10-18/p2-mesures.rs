// PROTOTYPE JETABLE (étape 2.9, P2) : mesures système et coût de la mesure elle-même.
use std::collections::HashMap;
use std::time::{Duration, Instant};
use sysinfo::{Components, Disks, ProcessesToUpdate, System, MINIMUM_CPU_UPDATE_INTERVAL};

fn own_rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    s.lines().find(|l| l.starts_with("VmRSS:")).and_then(|l| l.split_whitespace().nth(1)).and_then(|v| v.parse::<f64>().ok()).map(|k| k / 1024.0).unwrap_or(0.0)
}

fn main() {
    let rss0 = own_rss_mb();
    let t0 = Instant::now();
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let t_first = t0.elapsed();
    std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL.max(Duration::from_millis(500)));
    let t1 = Instant::now();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.refresh_cpu_usage();
    let t_second = t1.elapsed();

    println!("RAM totale {:.1} Go, utilisée {:.1} Go, swap {:.1}/{:.1} Go",
        sys.total_memory() as f64 / 1e9, sys.used_memory() as f64 / 1e9, sys.used_swap() as f64 / 1e9, sys.total_swap() as f64 / 1e9);
    println!("CPU global : {:.1} % ({} coeurs logiques)", sys.global_cpu_usage(), sys.cpus().len());

    // regroupement par logiciel (nom de l'exécutable) : RAM et CPU
    let mut groups: HashMap<String, (u64, f32, usize)> = HashMap::new();
    let mut threads = 0usize;
    for p in sys.processes().values() {
        if p.thread_kind().is_some() { threads += 1; continue; }
        let name = p.exe().and_then(|e| e.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| p.name().to_string_lossy().into_owned());
        let e = groups.entry(name).or_default();
        e.0 += p.memory();
        e.1 += p.cpu_usage();
        e.2 += 1;
    }
    let mut by_ram: Vec<_> = groups.iter().collect();
    by_ram.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    println!("--- Top 5 par RAM (regroupés par nom d'exécutable)");
    for (n, (m, c, k)) in by_ram.iter().take(5) { println!("{:<22} {:>7.0} Mo  {:>5.1} % CPU  {} processus", n, *m as f64 / 1e6, c, k); }
    let mut by_cpu: Vec<_> = groups.iter().collect();
    by_cpu.sort_by(|a, b| b.1 .1.partial_cmp(&a.1 .1).unwrap());
    println!("--- Top 3 par CPU");
    for (n, (m, c, k)) in by_cpu.iter().take(3) { println!("{:<22} {:>7.0} Mo  {:>5.1} % CPU  {} processus", n, *m as f64 / 1e6, c, k); }
    println!("{} entrées sysinfo dont {} fils d'exécution écartés ; {} logiciels distincts", sys.processes().len(), threads, groups.len());

    let disks = Disks::new_with_refreshed_list();
    println!("--- Disques");
    for d in disks.list().iter().filter(|d| d.total_space() > 1_000_000_000).take(4) {
        println!("{:<20} {:<8} {:.1} Go libres sur {:.1} Go", d.mount_point().display(), d.file_system().to_string_lossy(), d.available_space() as f64 / 1e9, d.total_space() as f64 / 1e9);
    }
    let comps = Components::new_with_refreshed_list();
    println!("--- Températures : {} capteurs", comps.list().len());
    for c in comps.list().iter().take(6) { println!("{:<30} {:?} °C", c.label(), c.temperature()); }

    println!("--- Coût de la mesure : 1er relevé {:?}, 2e relevé {:?}, RSS du prototype {:.1} Mo (avant {:.1} Mo)", t_first, t_second, own_rss_mb(), rss0);
}
